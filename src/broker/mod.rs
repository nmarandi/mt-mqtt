pub mod publisher;
pub mod subscriber;

use crate::session::SessionManager;
use crate::topic::TopicTree;
use std::collections::HashMap;
use tokio::sync::mpsc::{self, Receiver, Sender};

#[derive(Debug, Clone)]
pub struct PublishMessage {
    pub topic: String,
    pub payload: Vec<u8>,
    pub qos: u8,
    pub retain: bool,
}

#[derive(Debug)]
pub enum BrokerMessage {
    // Client connects with ID and clean_session flag
    Connect {
        client_id: String,
        clean_session: bool,
        sender: Sender<PublishMessage>,
        // Response channel to send back session_present flag
        response: tokio::sync::oneshot::Sender<bool>,
    },
    // Client disconnects
    Disconnect {
        client_id: String,
    },
    // Client subscribes to topics
    Subscribe {
        client_id: String,
        topics: Vec<String>,
    },
    // Client unsubscribes from topics
    Unsubscribe {
        client_id: String,
        topics: Vec<String>,
    },
    // Client publishes a message
    Publish(PublishMessage),
}


pub struct Broker {
    // Client ID -> message sender channel
    clients: HashMap<String, Sender<PublishMessage>>,
    // Topic subscription tree
    topic_tree: TopicTree,
    // Retained messages per topic
    retained_messages: HashMap<String, PublishMessage>,
    // Session manager for persistent sessions
    session_manager: SessionManager,
    // Channel for receiving broker commands
    sender: Sender<BrokerMessage>,
    receiver: Receiver<BrokerMessage>,
}

impl Broker {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(1000);

        Self {
            clients: HashMap::new(),
            topic_tree: TopicTree::new_root(),
            retained_messages: HashMap::new(),
            session_manager: SessionManager::new(),
            sender,
            receiver,
        }
    }

    pub async fn run(mut self) {
        tracing::info!("Broker started and ready to route messages");
        
        while let Some(message) = self.receiver.recv().await {
            match message {
                BrokerMessage::Connect { client_id, clean_session, sender, response } => {
                    tracing::info!("Broker: Client {} connected (clean_session={})", client_id, clean_session);
                    
                    // If clean_session=true, we need to clean up old subscriptions from topic tree
                    if clean_session {
                        if let Some(old_session) = self.session_manager.get_session(&client_id) {
                            // Remove old subscriptions from topic tree
                            for topic in &old_session.subscriptions {
                                self.topic_tree.unsubscribe(topic, &client_id);
                                tracing::debug!("Broker: Removed old subscription {} for client {} due to clean_session", topic, client_id);
                            }
                        }
                    }
                    
                    // Get or create session and determine if session was present
                    let (session_present, session) = self.session_manager
                        .get_or_create_session(client_id.clone(), clean_session).await;
                    
                    tracing::info!("Broker: Session present for client {}: {}", client_id, session_present);
                    
                    // Send response back to client
                    let _ = response.send(session_present);
                    
                    // Store client sender
                    self.clients.insert(client_id.clone(), sender.clone());
                    
                    // If session was present (persistent session), restore subscriptions
                    if session_present {
                        let subscriptions = session.subscriptions.clone();
                        for topic in &subscriptions {
                            self.topic_tree.subscribe(topic, &client_id);
                            tracing::debug!("Broker: Restored subscription for client {} to topic {}", client_id, topic);
                        }
                        
                        // Send any pending messages to the reconnected client
                        let pending_messages = session.take_pending_messages();
                        if !pending_messages.is_empty() {
                            tracing::info!("Broker: Delivering {} pending messages to client {}", 
                                pending_messages.len(), client_id);
                            
                            // Clear persisted messages after loading
                            let _ = self.session_manager.clear_persisted_messages(&client_id).await;
                            
                            for msg in pending_messages {
                                if let Some(client_sender) = self.clients.get(&client_id) {
                                    let _ = client_sender.send(msg).await;
                                }
                            }
                        }
                    }
                }
                
                BrokerMessage::Disconnect { client_id } => {
                    tracing::info!("Broker: Client {} disconnected", client_id);
                    
                    // Remove from active clients
                    self.clients.remove(&client_id);
                    
                    // If session is not persistent, clean it up
                    if let Some(session) = self.session_manager.get_session(&client_id) {
                        if !session.persistent {
                            // Clean session - remove subscriptions from topic tree
                            for topic in &session.subscriptions {
                                self.topic_tree.unsubscribe(topic, &client_id);
                            }
                            self.session_manager.remove_session(&client_id).await;
                            tracing::debug!("Broker: Cleaned up non-persistent session for client {}", client_id);
                        } else {
                            // Persist session state for persistent sessions
                            let _ = self.session_manager.persist_session(&client_id).await;
                            tracing::debug!("Broker: Keeping persistent session for client {}", client_id);
                        }
                    }
                }
                
                BrokerMessage::Subscribe { client_id, topics } => {
                    tracing::debug!("Broker: Client {} subscribing to {:?}", client_id, topics);
                    
                    // Add to topic tree
                    for topic in &topics {
                        self.topic_tree.subscribe(topic, &client_id);
                        
                        // Store subscription in session
                        if let Some(session) = self.session_manager.get_session_mut(&client_id) {
                            session.add_subscription(topic.clone());
                        }
                        
                        // Persist the session with new subscription
                        let _ = self.session_manager.persist_session(&client_id).await;
                        
                        // Send retained message if exists
                        if let Some(retained_msg) = self.retained_messages.get(topic) {
                            if let Some(client_sender) = self.clients.get(&client_id) {
                                let _ = client_sender.send(retained_msg.clone()).await;
                            }
                        }
                    }
                }
                
                BrokerMessage::Unsubscribe { client_id, topics } => {
                    tracing::debug!("Broker: Client {} unsubscribing from {:?}", client_id, topics);
                    
                    for topic in &topics {
                        self.topic_tree.unsubscribe(topic, &client_id);
                        
                        // Remove from session
                        if let Some(session) = self.session_manager.get_session_mut(&client_id) {
                            session.remove_subscription(topic);
                        }
                    }
                }
                
                BrokerMessage::Publish(msg) => {
                    tracing::debug!("Broker: Publishing to topic '{}', payload size: {} bytes", 
                            msg.topic, msg.payload.len());
                    
                    // Store retained message
                    if msg.retain {
                        if msg.payload.is_empty() {
                            self.retained_messages.remove(&msg.topic);
                        } else {
                            self.retained_messages.insert(msg.topic.clone(), msg.clone());
                        }
                    }
                    
                    // Find all subscribers matching this topic
                    let subscribers = self.find_subscribers(&msg.topic);
                    tracing::debug!("Broker: Found {} subscribers for topic '{}'", subscribers.len(), msg.topic);
                    
                    // Send message to all matching subscribers
                    for subscriber_id in subscribers {
                        if let Some(client_sender) = self.clients.get(&subscriber_id) {
                            // Client is online, send directly
                            let sender_clone = client_sender.clone();
                            let msg_clone = msg.clone();
                            let sub_id = subscriber_id.clone();
                            
                            // Spawn task to send message in parallel
                            tokio::spawn(async move {
                                match sender_clone.send(msg_clone).await {
                                    Ok(_) => tracing::trace!("Broker: Sent message to client {}", sub_id),
                                    Err(e) => tracing::debug!("Broker: Failed to send to {} (likely disconnected): {}", sub_id, e),
                                }
                            });
                        } else if let Some(session) = self.session_manager.get_session_mut(&subscriber_id) {
                            // Client is offline but has persistent session
                            if session.persistent && msg.qos > 0 {
                                // Queue message for offline delivery
                                session.queue_message(msg.clone());
                                
                                // Persist the queued message
                                let _ = self.session_manager.persist_queued_message(&subscriber_id, &msg).await;
                                
                                tracing::debug!("Broker: Queued message for offline client {} (QoS {})", 
                                    subscriber_id, msg.qos);
                            }
                        }
                    }
                }
            }
        }
    }

    fn find_subscribers(&self, topic: &str) -> Vec<String> {
        // Use topic tree to find matching subscribers
        // This handles wildcard matching (+, #)
        self.topic_tree.get_subscribers(topic)
    }

    pub fn get_sender(&self) -> Sender<BrokerMessage> {
        self.sender.clone()
    }
}