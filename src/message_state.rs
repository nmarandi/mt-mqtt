/// Message state tracking for QoS 1 and QoS 2 flows
use std::collections::HashMap;

#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields may be used for future retry/persistence features
pub enum MessageState {
    /// QoS 1: Waiting for PUBACK
    WaitingForPubAck { topic: String, payload: Vec<u8>, retain: bool },
    /// QoS 2: Waiting for PUBREC
    WaitingForPubRec { topic: String, payload: Vec<u8>, retain: bool },
    /// QoS 2: Waiting for PUBCOMP (after we sent PUBREL)
    WaitingForPubComp { topic: String },
    /// QoS 2: Received PUBREC, need to send PUBREL
    ReceivedPubRec { topic: String },
}

#[derive(Debug)]
pub struct MessageStateTracker {
    // packet_id -> message state
    states: HashMap<u16, MessageState>,
}

impl MessageStateTracker {
    pub fn new() -> Self {
        Self { states: HashMap::new() }
    }

    /// Track a new outgoing QoS 1 message
    pub fn track_qos1(&mut self, packet_id: u16, topic: String, payload: Vec<u8>, retain: bool) {
        self.states.insert(packet_id, MessageState::WaitingForPubAck { topic, payload, retain });
    }

    /// Track a new outgoing QoS 2 message
    pub fn track_qos2(&mut self, packet_id: u16, topic: String, payload: Vec<u8>, retain: bool) {
        self.states.insert(packet_id, MessageState::WaitingForPubRec { topic, payload, retain });
    }

    /// Handle PUBACK received (QoS 1 complete)
    pub fn handle_puback(&mut self, packet_id: u16) -> bool {
        if let Some(MessageState::WaitingForPubAck { .. }) = self.states.get(&packet_id) {
            self.states.remove(&packet_id);
            true
        } else {
            false
        }
    }

    /// Handle PUBREC received (QoS 2 step 1 complete, move to step 2)
    pub fn handle_pubrec(&mut self, packet_id: u16) -> bool {
        if let Some(state) = self.states.get(&packet_id) {
            match state {
                MessageState::WaitingForPubRec { topic, .. } => {
                    let topic = topic.clone();
                    self.states.insert(packet_id, MessageState::WaitingForPubComp { topic });
                    true
                }
                _ => false,
            }
        } else {
            false
        }
    }

    /// Handle PUBREL received (as server, for incoming QoS 2)
    pub fn handle_pubrel(&mut self, packet_id: u16) -> bool {
        if let Some(MessageState::ReceivedPubRec { .. }) = self.states.get(&packet_id) {
            self.states.remove(&packet_id);
            true
        } else {
            // Even if not tracked, send PUBCOMP
            true
        }
    }

    /// Handle PUBCOMP received (QoS 2 complete)
    pub fn handle_pubcomp(&mut self, packet_id: u16) -> bool {
        if let Some(MessageState::WaitingForPubComp { .. }) = self.states.get(&packet_id) {
            self.states.remove(&packet_id);
            true
        } else {
            false
        }
    }

    /// Mark that we received a QoS 2 PUBLISH and sent PUBREC, waiting for PUBREL
    pub fn mark_received_publish_qos2(&mut self, packet_id: u16, topic: String) {
        self.states.insert(packet_id, MessageState::ReceivedPubRec { topic });
    }

    /// Get the state for a packet ID
    #[allow(dead_code)]
    pub fn get_state(&self, packet_id: u16) -> Option<&MessageState> {
        self.states.get(&packet_id)
    }

    /// Remove a state (for cleanup)
    #[allow(dead_code)]
    pub fn remove(&mut self, packet_id: u16) -> Option<MessageState> {
        self.states.remove(&packet_id)
    }

    /// Get all tracked packet IDs
    #[allow(dead_code)]
    pub fn tracked_ids(&self) -> Vec<u16> {
        self.states.keys().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qos1_flow() {
        let mut tracker = MessageStateTracker::new();

        // Start QoS 1 publish
        tracker.track_qos1(1, "test/topic".to_string(), vec![1, 2, 3], false);
        assert!(tracker.get_state(1).is_some());

        // Receive PUBACK
        assert!(tracker.handle_puback(1));
        assert!(tracker.get_state(1).is_none());
    }

    #[test]
    fn test_qos2_flow() {
        let mut tracker = MessageStateTracker::new();

        // Start QoS 2 publish
        tracker.track_qos2(1, "test/topic".to_string(), vec![1, 2, 3], false);

        // Receive PUBREC
        assert!(tracker.handle_pubrec(1));

        // Should now be waiting for PUBCOMP
        match tracker.get_state(1) {
            Some(MessageState::WaitingForPubComp { .. }) => {}
            _ => panic!("Expected WaitingForPubComp state"),
        }

        // Receive PUBCOMP
        assert!(tracker.handle_pubcomp(1));
        assert!(tracker.get_state(1).is_none());
    }

    #[test]
    fn test_server_qos2_flow() {
        let mut tracker = MessageStateTracker::new();

        // Receive QoS 2 PUBLISH, send PUBREC
        tracker.mark_received_publish_qos2(1, "test/topic".to_string());

        // Receive PUBREL
        assert!(tracker.handle_pubrel(1));
        assert!(tracker.get_state(1).is_none());
    }
}
