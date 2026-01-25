use mt_mqtt::broker::{Broker, BrokerMessage, PublishMessage};
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn test_broker_routing() {
    // Create broker
    let broker = Broker::new();
    let broker_tx = broker.get_sender();
    let (client_tx, mut client_rx) = mpsc::channel(100);
    
    // Spawn broker task
    tokio::spawn(async move {
        broker.run().await;
    });
    
    // Connect client
    broker_tx.send(BrokerMessage::Connect {
        client_id: "test_client".to_string(),
        sender: client_tx.clone(),
    }).await.unwrap();
    
    // Give broker time to process
    tokio::time::sleep(Duration::from_millis(10)).await;
    
    // Subscribe to topic
    broker_tx.send(BrokerMessage::Subscribe {
        client_id: "test_client".to_string(),
        topics: vec!["test/topic".to_string()],
    }).await.unwrap();
    
    // Give broker time to process
    tokio::time::sleep(Duration::from_millis(10)).await;
    
    // Publish message
    broker_tx.send(BrokerMessage::Publish(PublishMessage {
        topic: "test/topic".to_string(),
        payload: b"test message".to_vec(),
        qos: 0,
        retain: false,
    })).await.unwrap();
    
    // Should receive the published message
    let msg = timeout(Duration::from_millis(100), client_rx.recv())
        .await
        .expect("Should receive message")
        .expect("Message should not be None");
    
    assert_eq!(msg.topic, "test/topic");
    assert_eq!(msg.payload, b"test message");
}

#[tokio::test]
async fn test_wildcard_routing() {
    let broker = Broker::new();
    let broker_tx = broker.get_sender();
    let (client_tx, mut client_rx) = mpsc::channel(100);
    
    // Spawn broker task
    tokio::spawn(async move {
        broker.run().await;
    });
    
    // Connect client
    broker_tx.send(BrokerMessage::Connect {
        client_id: "wildcard_client".to_string(),
        sender: client_tx.clone(),
    }).await.unwrap();
    
    tokio::time::sleep(Duration::from_millis(10)).await;
    
    // Subscribe to wildcard
    broker_tx.send(BrokerMessage::Subscribe {
        client_id: "wildcard_client".to_string(),
        topics: vec!["sensor/#".to_string()],
    }).await.unwrap();
    
    tokio::time::sleep(Duration::from_millis(10)).await;
    
    // Publish to matching topic
    broker_tx.send(BrokerMessage::Publish(PublishMessage {
        topic: "sensor/temperature".to_string(),
        payload: b"25.5".to_vec(),
        qos: 0,
        retain: false,
    })).await.unwrap();
    
    // Should receive the message
    let msg = timeout(Duration::from_millis(100), client_rx.recv())
        .await
        .expect("Should receive wildcard matched message")
        .expect("Message should not be None");
    
    assert_eq!(msg.topic, "sensor/temperature");
    assert_eq!(msg.payload, b"25.5");
}

#[tokio::test]
async fn test_multiple_subscribers() {
    let broker = Broker::new();
    let broker_tx = broker.get_sender();
    let (client1_tx, mut client1_rx) = mpsc::channel(100);
    let (client2_tx, mut client2_rx) = mpsc::channel(100);
    
    // Spawn broker task
    tokio::spawn(async move {
        broker.run().await;
    });
    
    // Connect two clients
    broker_tx.send(BrokerMessage::Connect {
        client_id: "client1".to_string(),
        sender: client1_tx,
    }).await.unwrap();
    
    broker_tx.send(BrokerMessage::Connect {
        client_id: "client2".to_string(),
        sender: client2_tx,
    }).await.unwrap();
    
    tokio::time::sleep(Duration::from_millis(10)).await;
    
    // Both subscribe to same topic
    broker_tx.send(BrokerMessage::Subscribe {
        client_id: "client1".to_string(),
        topics: vec!["broadcast".to_string()],
    }).await.unwrap();
    
    broker_tx.send(BrokerMessage::Subscribe {
        client_id: "client2".to_string(),
        topics: vec!["broadcast".to_string()],
    }).await.unwrap();
    
    tokio::time::sleep(Duration::from_millis(10)).await;
    
    // Publish message
    broker_tx.send(BrokerMessage::Publish(PublishMessage {
        topic: "broadcast".to_string(),
        payload: b"message to all".to_vec(),
        qos: 0,
        retain: false,
    })).await.unwrap();
    
    // Both should receive
    let msg1 = timeout(Duration::from_millis(100), client1_rx.recv())
        .await
        .expect("Client 1 should receive")
        .expect("Message should not be None");
    
    let msg2 = timeout(Duration::from_millis(100), client2_rx.recv())
        .await
        .expect("Client 2 should receive")
        .expect("Message should not be None");
    
    assert_eq!(msg1.topic, "broadcast");
    assert_eq!(msg2.topic, "broadcast");
}

// Test for unsubscribe functionality
#[tokio::test]
async fn test_unsubscribe() {
    let broker = Broker::new();
    let broker_tx = broker.get_sender();
    let (client_tx, mut client_rx) = mpsc::channel(100);
    
    // Spawn broker task
    tokio::spawn(async move {
        broker.run().await;
    });
    
    // Connect client
    broker_tx.send(BrokerMessage::Connect {
        client_id: "test_client".to_string(),
        sender: client_tx.clone(),
    }).await.unwrap();
    
    tokio::time::sleep(Duration::from_millis(10)).await;
    
    // Subscribe to topic
    broker_tx.send(BrokerMessage::Subscribe {
        client_id: "test_client".to_string(),
        topics: vec!["test/topic".to_string()],
    }).await.unwrap();
    
    tokio::time::sleep(Duration::from_millis(10)).await;
    
    // Publish message - should receive
    broker_tx.send(BrokerMessage::Publish(PublishMessage {
        topic: "test/topic".to_string(),
        payload: b"message 1".to_vec(),
        qos: 0,
        retain: false,
    })).await.unwrap();
    
    let msg1 = timeout(Duration::from_millis(100), client_rx.recv())
        .await
        .expect("Should receive first message")
        .expect("Message should not be None");
    assert_eq!(msg1.payload, b"message 1");
    
    // Unsubscribe
    broker_tx.send(BrokerMessage::Unsubscribe {
        client_id: "test_client".to_string(),
        topics: vec!["test/topic".to_string()],
    }).await.unwrap();
    
    tokio::time::sleep(Duration::from_millis(10)).await;
    
    // Publish another message - should NOT receive
    broker_tx.send(BrokerMessage::Publish(PublishMessage {
        topic: "test/topic".to_string(),
        payload: b"message 2".to_vec(),
        qos: 0,
        retain: false,
    })).await.unwrap();
    
    // Should timeout (no message received)
    let result = timeout(Duration::from_millis(100), client_rx.recv()).await;
    assert!(result.is_err(), "Should not receive message after unsubscribe");
}

// Test for retained messages
#[tokio::test]
async fn test_retained_messages() {
    let broker = Broker::new();
    let broker_tx = broker.get_sender();
    let (client_tx, mut client_rx) = mpsc::channel(100);
    
    // Spawn broker task
    tokio::spawn(async move {
        broker.run().await;
    });
    
    // Publish retained message BEFORE subscription
    broker_tx.send(BrokerMessage::Publish(PublishMessage {
        topic: "sensor/temperature".to_string(),
        payload: b"23.5".to_vec(),
        qos: 0,
        retain: true,
    })).await.unwrap();
    
    tokio::time::sleep(Duration::from_millis(10)).await;
    
    // Now connect and subscribe
    broker_tx.send(BrokerMessage::Connect {
        client_id: "late_client".to_string(),
        sender: client_tx.clone(),
    }).await.unwrap();
    
    tokio::time::sleep(Duration::from_millis(10)).await;
    
    broker_tx.send(BrokerMessage::Subscribe {
        client_id: "late_client".to_string(),
        topics: vec!["sensor/temperature".to_string()],
    }).await.unwrap();
    
    // Should receive the retained message
    let msg = timeout(Duration::from_millis(100), client_rx.recv())
        .await
        .expect("Should receive retained message")
        .expect("Message should not be None");
    
    assert_eq!(msg.topic, "sensor/temperature");
    assert_eq!(msg.payload, b"23.5");
}

// Test for clearing retained message
#[tokio::test]
async fn test_clear_retained_message() {
    let broker = Broker::new();
    let broker_tx = broker.get_sender();
    
    // Spawn broker task
    tokio::spawn(async move {
        broker.run().await;
    });
    
    // Publish retained message
    broker_tx.send(BrokerMessage::Publish(PublishMessage {
        topic: "sensor/temperature".to_string(),
        payload: b"23.5".to_vec(),
        qos: 0,
        retain: true,
    })).await.unwrap();
    
    tokio::time::sleep(Duration::from_millis(10)).await;
    
    // Clear retained message with empty payload
    broker_tx.send(BrokerMessage::Publish(PublishMessage {
        topic: "sensor/temperature".to_string(),
        payload: vec![],
        qos: 0,
        retain: true,
    })).await.unwrap();
    
    tokio::time::sleep(Duration::from_millis(10)).await;
    
    // Subscribe after clearing
    let (client_tx, mut client_rx) = mpsc::channel(100);
    broker_tx.send(BrokerMessage::Connect {
        client_id: "test_client".to_string(),
        sender: client_tx.clone(),
    }).await.unwrap();
    
    tokio::time::sleep(Duration::from_millis(10)).await;
    
    broker_tx.send(BrokerMessage::Subscribe {
        client_id: "test_client".to_string(),
        topics: vec!["sensor/temperature".to_string()],
    }).await.unwrap();
    
    // Should NOT receive any message (retained was cleared)
    let result = timeout(Duration::from_millis(100), client_rx.recv()).await;
    assert!(result.is_err(), "Should not receive cleared retained message");
}

#[tokio::test]
async fn test_qos1_publish() {
    // Create broker
    let broker = Broker::new();
    let broker_tx = broker.get_sender();
    let (client_tx, mut client_rx) = mpsc::channel(100);
    
    // Spawn broker task
    tokio::spawn(async move {
        broker.run().await;
    });
    
    // Connect client
    broker_tx.send(BrokerMessage::Connect {
        client_id: "test_client".to_string(),
        sender: client_tx.clone(),
    }).await.unwrap();
    
    tokio::time::sleep(Duration::from_millis(10)).await;
    
    // Subscribe to topic
    broker_tx.send(BrokerMessage::Subscribe {
        client_id: "test_client".to_string(),
        topics: vec!["qos/test".to_string()],
    }).await.unwrap();
    
    tokio::time::sleep(Duration::from_millis(10)).await;
    
    // Publish QoS 1 message
    broker_tx.send(BrokerMessage::Publish(PublishMessage {
        topic: "qos/test".to_string(),
        payload: b"QoS 1 message".to_vec(),
        qos: 1,
        retain: false,
    })).await.unwrap();
    
    // Should receive the message with QoS 1
    let msg = timeout(Duration::from_millis(100), client_rx.recv())
        .await
        .expect("Should receive message")
        .expect("Message should not be None");
    
    assert_eq!(msg.topic, "qos/test");
    assert_eq!(msg.payload, b"QoS 1 message");
    assert_eq!(msg.qos, 1);
}

#[tokio::test]
async fn test_qos2_publish() {
    // Create broker
    let broker = Broker::new();
    let broker_tx = broker.get_sender();
    let (client_tx, mut client_rx) = mpsc::channel(100);
    
    // Spawn broker task
    tokio::spawn(async move {
        broker.run().await;
    });
    
    // Connect client
    broker_tx.send(BrokerMessage::Connect {
        client_id: "test_client".to_string(),
        sender: client_tx.clone(),
    }).await.unwrap();
    
    tokio::time::sleep(Duration::from_millis(10)).await;
    
    // Subscribe to topic
    broker_tx.send(BrokerMessage::Subscribe {
        client_id: "test_client".to_string(),
        topics: vec!["qos2/test".to_string()],
    }).await.unwrap();
    
    tokio::time::sleep(Duration::from_millis(10)).await;
    
    // Publish QoS 2 message
    broker_tx.send(BrokerMessage::Publish(PublishMessage {
        topic: "qos2/test".to_string(),
        payload: b"QoS 2 message".to_vec(),
        qos: 2,
        retain: false,
    })).await.unwrap();
    
    // Should receive the message with QoS 2
    let msg = timeout(Duration::from_millis(100), client_rx.recv())
        .await
        .expect("Should receive message")
        .expect("Message should not be None");
    
    assert_eq!(msg.topic, "qos2/test");
    assert_eq!(msg.payload, b"QoS 2 message");
    assert_eq!(msg.qos, 2);
}
