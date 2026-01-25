pub mod publisher;
pub mod subscriber;

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
    // Client connects with ID
    Connect {
        client_id: String,
        sender: Sender<PublishMessage>,
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
            sender,
            receiver,
        }
    }

    pub async fn run(mut self) {
        tracing::info!("Broker started and ready to route messages");
        
        while let Some(message) = self.receiver.recv().await {
            match message {
                BrokerMessage::Connect { client_id, sender } => {
                    tracing::info!("Broker: Client {} connected", client_id);
                    self.clients.insert(client_id, sender);
                }
                
                BrokerMessage::Disconnect { client_id } => {
                    tracing::info!("Broker: Client {} disconnected", client_id);
                    self.clients.remove(&client_id);
                }
                
                BrokerMessage::Subscribe { client_id, topics } => {
                    tracing::debug!("Broker: Client {} subscribing to {:?}", client_id, topics);
                    for topic in topics {
                        self.topic_tree.subscribe(&topic, &client_id);
                        
                        // Send retained message if exists
                        if let Some(retained_msg) = self.retained_messages.get(&topic) {
                            if let Some(client_sender) = self.clients.get(&client_id) {
                                let _ = client_sender.send(retained_msg.clone()).await;
                            }
                        }
                    }
                }
                
                BrokerMessage::Unsubscribe { client_id, topics } => {
                    tracing::debug!("Broker: Client {} unsubscribing from {:?}", client_id, topics);
                    for topic in topics {
                        self.topic_tree.unsubscribe(&topic, &client_id);
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
                    
                    // Send message to all matching subscribers in parallel
                    for subscriber_id in subscribers {
                        if let Some(client_sender) = self.clients.get(&subscriber_id) {
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