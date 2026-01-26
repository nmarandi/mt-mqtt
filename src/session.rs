use crate::{
    broker::PublishMessage,
    message_state::MessageStateTracker,
    packet_id::PacketIdManager,
};
use std::collections::HashMap;

/// Represents a persistent MQTT session for a client
#[derive(Debug)]
pub struct Session {
    /// Client ID for this session
    pub client_id: String,
    /// Whether this session should be persisted (clean_session = false)
    pub persistent: bool,
    /// List of topic subscriptions for this client
    pub subscriptions: Vec<String>,
    /// Queue of pending messages for offline client (QoS 1 and 2 only)
    pub pending_messages: Vec<PublishMessage>,
    /// Packet ID manager for this session
    pub packet_id_manager: PacketIdManager,
    /// Message state tracker for QoS flows
    pub message_state_tracker: MessageStateTracker,
}

impl Session {
    pub fn new(client_id: String, persistent: bool) -> Self {
        Self {
            client_id,
            persistent,
            subscriptions: Vec::new(),
            pending_messages: Vec::new(),
            packet_id_manager: PacketIdManager::new(),
            message_state_tracker: MessageStateTracker::new(),
        }
    }

    /// Add a subscription to this session
    pub fn add_subscription(&mut self, topic: String) {
        if !self.subscriptions.contains(&topic) {
            self.subscriptions.push(topic);
        }
    }

    /// Remove a subscription from this session
    pub fn remove_subscription(&mut self, topic: &str) {
        self.subscriptions.retain(|t| t != topic);
    }

    /// Queue a message for offline delivery (QoS 1 and 2 only)
    pub fn queue_message(&mut self, message: PublishMessage) {
        // Only queue QoS 1 and 2 messages
        if message.qos > 0 {
            self.pending_messages.push(message);
        }
    }

    /// Clear all pending messages
    pub fn clear_pending_messages(&mut self) {
        self.pending_messages.clear();
    }

    /// Get and clear all pending messages (for delivery on reconnect)
    pub fn take_pending_messages(&mut self) -> Vec<PublishMessage> {
        std::mem::take(&mut self.pending_messages)
    }
}

/// Manages all client sessions
#[derive(Debug, Default)]
pub struct SessionManager {
    /// Map of client_id to Session
    sessions: HashMap<String, Session>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    /// Create or retrieve a session
    /// Returns (session_present, mutable reference to session)
    pub fn get_or_create_session(&mut self, client_id: String, clean_session: bool) -> (bool, &mut Session) {
        let session_present = if clean_session {
            // Clean session requested - remove any existing session
            self.sessions.remove(&client_id);
            false
        } else {
            // Persistent session requested - check if session exists
            self.sessions.contains_key(&client_id)
        };

        // Get or create session
        let session = self.sessions.entry(client_id.clone())
            .or_insert_with(|| Session::new(client_id, !clean_session));

        // Update persistent flag in case it changed
        session.persistent = !clean_session;

        (session_present, session)
    }

    /// Get a session by client_id
    pub fn get_session(&self, client_id: &str) -> Option<&Session> {
        self.sessions.get(client_id)
    }

    /// Get a mutable session by client_id
    pub fn get_session_mut(&mut self, client_id: &str) -> Option<&mut Session> {
        self.sessions.get_mut(client_id)
    }

    /// Remove a session (on clean disconnect or clean_session=true reconnect)
    pub fn remove_session(&mut self, client_id: &str) {
        self.sessions.remove(client_id);
    }

    /// Check if a session exists
    pub fn has_session(&self, client_id: &str) -> bool {
        self.sessions.contains_key(client_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = Session::new("client1".to_string(), true);
        assert_eq!(session.client_id, "client1");
        assert!(session.persistent);
        assert!(session.subscriptions.is_empty());
        assert!(session.pending_messages.is_empty());
    }

    #[test]
    fn test_session_subscriptions() {
        let mut session = Session::new("client1".to_string(), true);
        session.add_subscription("topic/1".to_string());
        session.add_subscription("topic/2".to_string());
        assert_eq!(session.subscriptions.len(), 2);
        
        // Adding duplicate should not increase count
        session.add_subscription("topic/1".to_string());
        assert_eq!(session.subscriptions.len(), 2);
        
        session.remove_subscription("topic/1");
        assert_eq!(session.subscriptions.len(), 1);
        assert_eq!(session.subscriptions[0], "topic/2");
    }

    #[test]
    fn test_session_message_queueing() {
        let mut session = Session::new("client1".to_string(), true);
        
        // QoS 0 should not be queued
        let msg_qos0 = PublishMessage {
            topic: "test".to_string(),
            payload: vec![1, 2, 3],
            qos: 0,
            retain: false,
        };
        session.queue_message(msg_qos0);
        assert_eq!(session.pending_messages.len(), 0);
        
        // QoS 1 should be queued
        let msg_qos1 = PublishMessage {
            topic: "test".to_string(),
            payload: vec![1, 2, 3],
            qos: 1,
            retain: false,
        };
        session.queue_message(msg_qos1);
        assert_eq!(session.pending_messages.len(), 1);
        
        // QoS 2 should be queued
        let msg_qos2 = PublishMessage {
            topic: "test".to_string(),
            payload: vec![4, 5, 6],
            qos: 2,
            retain: false,
        };
        session.queue_message(msg_qos2);
        assert_eq!(session.pending_messages.len(), 2);
    }

    #[test]
    fn test_session_manager_clean_session() {
        let mut manager = SessionManager::new();
        
        // First connection with clean_session=true
        let (session_present, _session) = manager.get_or_create_session("client1".to_string(), true);
        assert!(!session_present);
        
        // Reconnect with clean_session=true should not have session present
        let (session_present, _session) = manager.get_or_create_session("client1".to_string(), true);
        assert!(!session_present);
    }

    #[test]
    fn test_session_manager_persistent_session() {
        let mut manager = SessionManager::new();
        
        // First connection with clean_session=false
        let (session_present, session) = manager.get_or_create_session("client1".to_string(), false);
        assert!(!session_present); // First time, no session exists
        session.add_subscription("topic/1".to_string());
        
        // Reconnect with clean_session=false should restore session
        let (session_present, session) = manager.get_or_create_session("client1".to_string(), false);
        assert!(session_present); // Session should be present
        assert_eq!(session.subscriptions.len(), 1);
        assert_eq!(session.subscriptions[0], "topic/1");
    }

    #[test]
    fn test_session_manager_clean_clears_persistent() {
        let mut manager = SessionManager::new();
        
        // Create persistent session
        let (_session_present, session) = manager.get_or_create_session("client1".to_string(), false);
        session.add_subscription("topic/1".to_string());
        
        // Reconnect with clean_session=true should clear session
        let (session_present, session) = manager.get_or_create_session("client1".to_string(), true);
        assert!(!session_present);
        assert!(session.subscriptions.is_empty());
    }

    #[test]
    fn test_take_pending_messages() {
        let mut session = Session::new("client1".to_string(), true);
        
        let msg1 = PublishMessage {
            topic: "test".to_string(),
            payload: vec![1, 2, 3],
            qos: 1,
            retain: false,
        };
        let msg2 = PublishMessage {
            topic: "test".to_string(),
            payload: vec![4, 5, 6],
            qos: 2,
            retain: false,
        };
        
        session.queue_message(msg1);
        session.queue_message(msg2);
        assert_eq!(session.pending_messages.len(), 2);
        
        let messages = session.take_pending_messages();
        assert_eq!(messages.len(), 2);
        assert_eq!(session.pending_messages.len(), 0);
    }
}
