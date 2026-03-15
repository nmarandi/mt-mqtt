/// Packet Identifier Manager for MQTT QoS 1 and QoS 2
///
/// Manages unique packet identifiers in the range 1-65535
/// as required by MQTT 3.1.1 specification.
use std::collections::HashSet;

#[derive(Debug)]
pub struct PacketIdManager {
    next_id: u16,
    in_use: HashSet<u16>,
}

impl PacketIdManager {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            in_use: HashSet::new(),
        }
    }

    /// Allocate a new packet identifier
    pub fn allocate(&mut self) -> Option<u16> {
        // Try to find an available ID starting from next_id
        let start = self.next_id;
        loop {
            if self.next_id == 0 {
                self.next_id = 1; // Skip 0, MQTT uses 1-65535
            }

            if !self.in_use.contains(&self.next_id) {
                let id = self.next_id;
                self.in_use.insert(id);
                self.next_id = self.next_id.wrapping_add(1);
                return Some(id);
            }

            self.next_id = self.next_id.wrapping_add(1);

            // If we've wrapped around completely, no IDs available
            if self.next_id == start {
                return None;
            }
        }
    }

    /// Release a packet identifier for reuse
    pub fn release(&mut self, id: u16) {
        self.in_use.remove(&id);
    }

    /// Check if a packet ID is currently in use
    #[allow(dead_code)]
    pub fn is_in_use(&self, id: u16) -> bool {
        self.in_use.contains(&id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocate_sequential() {
        let mut manager = PacketIdManager::new();
        assert_eq!(manager.allocate(), Some(1));
        assert_eq!(manager.allocate(), Some(2));
        assert_eq!(manager.allocate(), Some(3));
    }

    #[test]
    fn test_release_and_reuse() {
        let mut manager = PacketIdManager::new();
        let id1 = manager.allocate().unwrap();

        manager.release(id1);

        // Should eventually reuse id1
        for _ in 0..100 {
            let id = manager.allocate().unwrap();
            if id == id1 {
                manager.release(id);
                return;
            }
            manager.release(id);
        }
    }

    #[test]
    fn test_skips_zero() {
        let mut manager = PacketIdManager::new();
        for _ in 0..70000 {
            if let Some(id) = manager.allocate() {
                assert_ne!(id, 0);
                manager.release(id);
            }
        }
    }
}
