#[cfg(test)]
mod tests {
    use super::super::TopicTree;

    #[test]
    fn test_exact_topic_match() {
        let mut tree = TopicTree::new_root();
        tree.subscribe("test/topic", "client1");
        
        let subscribers = tree.get_subscribers("test/topic");
        assert_eq!(subscribers.len(), 1);
        assert!(subscribers.contains(&"client1".to_string()));
    }

    #[test]
    fn test_multilevel_wildcard() {
        let mut tree = TopicTree::new_root();
        tree.subscribe("sensor/#", "client1");
        
        // Should match all these
        assert_eq!(tree.get_subscribers("sensor/temp").len(), 1);
        assert_eq!(tree.get_subscribers("sensor/humidity/room1").len(), 1);
        assert_eq!(tree.get_subscribers("sensor/a/b/c/d").len(), 1);
        
        // Should NOT match this
        assert_eq!(tree.get_subscribers("device/temp").len(), 0);
    }

    #[test]
    fn test_single_level_wildcard() {
        let mut tree = TopicTree::new_root();
        tree.subscribe("device/+/status", "client1");
        
        // Should match
        assert_eq!(tree.get_subscribers("device/123/status").len(), 1);
        assert_eq!(tree.get_subscribers("device/abc/status").len(), 1);
        
        // Should NOT match (too many levels)
        assert_eq!(tree.get_subscribers("device/123/456/status").len(), 0);
        
        // Should NOT match (different topic)
        assert_eq!(tree.get_subscribers("device/123/temp").len(), 0);
    }

    #[test]
    fn test_multiple_subscribers_same_topic() {
        let mut tree = TopicTree::new_root();
        tree.subscribe("test/topic", "client1");
        tree.subscribe("test/topic", "client2");
        tree.subscribe("test/topic", "client3");
        
        let subscribers = tree.get_subscribers("test/topic");
        assert_eq!(subscribers.len(), 3);
    }

    // TODO: Implement unsubscribe functionality
    // #[test]
    // fn test_unsubscribe() {
    //     let tree = TopicTree::new_root();
    //     tree.subscribe("test/topic", "client1");
    //     tree.subscribe("test/topic", "client2");
    //     
    //     tree.unsubscribe("test/topic", "client1");
    //     
    //     let subscribers = tree.get_subscribers("test/topic");
    //     assert_eq!(subscribers.len(), 1);
    //     assert!(!subscribers.contains(&"client1".to_string()));
    //     assert!(subscribers.contains(&"client2".to_string()));
    // }

    #[test]
    fn test_unsubscribe_basic() {
        let mut tree = TopicTree::new_root();
        tree.subscribe("test/topic", "client1");
        tree.subscribe("test/topic", "client2");
        
        tree.unsubscribe("test/topic", "client1");
        
        let subscribers = tree.get_subscribers("test/topic");
        assert_eq!(subscribers.len(), 1);
        assert!(!subscribers.contains(&"client1".to_string()));
        assert!(subscribers.contains(&"client2".to_string()));
    }

    #[test]
    fn test_unsubscribe_multilevel_wildcard() {
        let mut tree = TopicTree::new_root();
        tree.subscribe("sensor/#", "client1");
        tree.subscribe("sensor/temperature", "client2");
        
        // Client1 should receive from sensor/temperature
        assert_eq!(tree.get_subscribers("sensor/temperature").len(), 2);
        
        tree.unsubscribe("sensor/#", "client1");
        
        // Only client2 should remain
        let subscribers = tree.get_subscribers("sensor/temperature");
        assert_eq!(subscribers.len(), 1);
        assert!(!subscribers.contains(&"client1".to_string()));
        assert!(subscribers.contains(&"client2".to_string()));
    }

    #[test]
    fn test_unsubscribe_singlelevel_wildcard() {
        let mut tree = TopicTree::new_root();
        tree.subscribe("sensor/+", "client1");
        tree.subscribe("sensor/temperature", "client2");
        
        assert_eq!(tree.get_subscribers("sensor/temperature").len(), 2);
        
        tree.unsubscribe("sensor/+", "client1");
        
        let subscribers = tree.get_subscribers("sensor/temperature");
        assert_eq!(subscribers.len(), 1);
        assert!(!subscribers.contains(&"client1".to_string()));
    }

    #[test]
    fn test_unsubscribe_root_wildcard() {
        let mut tree = TopicTree::new_root();
        tree.subscribe("#", "client1");
        tree.subscribe("test/topic", "client2");
        
        // Both should receive
        assert_eq!(tree.get_subscribers("test/topic").len(), 2);
        
        tree.unsubscribe("#", "client1");
        
        // Only client2 should remain
        let subscribers = tree.get_subscribers("test/topic");
        assert_eq!(subscribers.len(), 1);
        assert!(subscribers.contains(&"client2".to_string()));
    }

    #[test]
    fn test_unsubscribe_nonexistent() {
        let mut tree = TopicTree::new_root();
        tree.subscribe("test/topic", "client1");
        
        // Unsubscribe a client that doesn't exist - should not panic
        tree.unsubscribe("test/topic", "client2");
        tree.unsubscribe("different/topic", "client1");
        
        // Original subscription should still be there
        let subscribers = tree.get_subscribers("test/topic");
        assert_eq!(subscribers.len(), 1);
        assert!(subscribers.contains(&"client1".to_string()));
    }

    // Mixed wildcards like "sensor/+/data/#" are now supported!
    #[test]
    fn test_mixed_wildcards() {
        let mut tree = TopicTree::new_root();
        tree.subscribe("sensor/+/data/#", "client1");
        
        // Should match
        assert_eq!(tree.get_subscribers("sensor/temp/data/avg").len(), 1);
        assert_eq!(tree.get_subscribers("sensor/humidity/data/min/max").len(), 1);
        
        // Should NOT match (missing middle level)
        assert_eq!(tree.get_subscribers("sensor/data/avg").len(), 0);
    }

    #[test]
    fn test_root_wildcard() {
        let mut tree = TopicTree::new_root();
        tree.subscribe("#", "client1");
        
        // Should match everything
        assert_eq!(tree.get_subscribers("any").len(), 1);
        assert_eq!(tree.get_subscribers("any/topic").len(), 1);
        assert_eq!(tree.get_subscribers("any/topic/at/all").len(), 1);
    }

    #[test]
    fn test_empty_topic() {
        let tree = TopicTree::new_root();
        
        // Empty topic should return no subscribers
        assert_eq!(tree.get_subscribers("").len(), 0);
    }

    #[test]
    fn test_case_sensitive() {
        let mut tree = TopicTree::new_root();
        tree.subscribe("Test/Topic", "client1");
        
        // MQTT topics are case-sensitive
        assert_eq!(tree.get_subscribers("Test/Topic").len(), 1);
        assert_eq!(tree.get_subscribers("test/topic").len(), 0);
        assert_eq!(tree.get_subscribers("TEST/TOPIC").len(), 0);
    }

    #[test]
    fn test_overlapping_subscriptions() {
        let mut tree = TopicTree::new_root();
        tree.subscribe("sensor/#", "client1");
        tree.subscribe("sensor/temp", "client2");
        tree.subscribe("sensor/+/data", "client3");
        
        // "sensor/temp" should match client1 and client2
        let subs = tree.get_subscribers("sensor/temp");
        assert_eq!(subs.len(), 2);
        
        // "sensor/temp/data" should match client1 and client3
        let subs = tree.get_subscribers("sensor/temp/data");
        assert_eq!(subs.len(), 2);
    }
}
