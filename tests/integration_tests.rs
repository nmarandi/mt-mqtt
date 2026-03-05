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
    let (conn_tx, _conn_rx) = tokio::sync::oneshot::channel();
    broker_tx
        .send(BrokerMessage::Connect {
            client_id: "test_client".to_string(),
            sender: client_tx.clone(),
            clean_session: true,
            response: conn_tx,
            will_message: None,
        })
        .await
        .unwrap();

    // Give broker time to process
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Subscribe to topic
    broker_tx
        .send(BrokerMessage::Subscribe {
            client_id: "test_client".to_string(),
            topics: vec!["test/topic".to_string()],
        })
        .await
        .unwrap();

    // Give broker time to process
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Publish message
    broker_tx
        .send(BrokerMessage::Publish(PublishMessage {
            topic: "test/topic".to_string(),
            payload: b"test message".to_vec(),
            qos: 0,
            retain: false,
        }))
        .await
        .unwrap();

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
    let (conn_tx, _conn_rx) = tokio::sync::oneshot::channel();
    broker_tx
        .send(BrokerMessage::Connect {
            client_id: "wildcard_client".to_string(),
            sender: client_tx.clone(),
            clean_session: true,
            response: conn_tx,
            will_message: None,
        })
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Subscribe to wildcard
    broker_tx
        .send(BrokerMessage::Subscribe {
            client_id: "wildcard_client".to_string(),
            topics: vec!["sensor/#".to_string()],
        })
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Publish to matching topic
    broker_tx
        .send(BrokerMessage::Publish(PublishMessage {
            topic: "sensor/temperature".to_string(),
            payload: b"25.5".to_vec(),
            qos: 0,
            retain: false,
        }))
        .await
        .unwrap();

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
    let (conn_tx, _conn_rx) = tokio::sync::oneshot::channel();
    broker_tx
        .send(BrokerMessage::Connect {
            client_id: "client1".to_string(),
            sender: client1_tx,
            clean_session: true,
            response: conn_tx,
            will_message: None,
        })
        .await
        .unwrap();

    let (conn_tx, _conn_rx) = tokio::sync::oneshot::channel();
    broker_tx
        .send(BrokerMessage::Connect {
            client_id: "client2".to_string(),
            sender: client2_tx,
            clean_session: true,
            response: conn_tx,
            will_message: None,
        })
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Both subscribe to same topic
    broker_tx
        .send(BrokerMessage::Subscribe {
            client_id: "client1".to_string(),
            topics: vec!["broadcast".to_string()],
        })
        .await
        .unwrap();

    broker_tx
        .send(BrokerMessage::Subscribe {
            client_id: "client2".to_string(),
            topics: vec!["broadcast".to_string()],
        })
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Publish message
    broker_tx
        .send(BrokerMessage::Publish(PublishMessage {
            topic: "broadcast".to_string(),
            payload: b"message to all".to_vec(),
            qos: 0,
            retain: false,
        }))
        .await
        .unwrap();

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
    let (conn_tx, _conn_rx) = tokio::sync::oneshot::channel();
    broker_tx
        .send(BrokerMessage::Connect {
            client_id: "test_client".to_string(),
            sender: client_tx.clone(),
            clean_session: true,
            response: conn_tx,
            will_message: None,
        })
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Subscribe to topic
    broker_tx
        .send(BrokerMessage::Subscribe {
            client_id: "test_client".to_string(),
            topics: vec!["test/topic".to_string()],
        })
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Publish message - should receive
    broker_tx
        .send(BrokerMessage::Publish(PublishMessage {
            topic: "test/topic".to_string(),
            payload: b"message 1".to_vec(),
            qos: 0,
            retain: false,
        }))
        .await
        .unwrap();

    let msg1 = timeout(Duration::from_millis(100), client_rx.recv())
        .await
        .expect("Should receive first message")
        .expect("Message should not be None");
    assert_eq!(msg1.payload, b"message 1");

    // Unsubscribe
    broker_tx
        .send(BrokerMessage::Unsubscribe {
            client_id: "test_client".to_string(),
            topics: vec!["test/topic".to_string()],
        })
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Publish another message - should NOT receive
    broker_tx
        .send(BrokerMessage::Publish(PublishMessage {
            topic: "test/topic".to_string(),
            payload: b"message 2".to_vec(),
            qos: 0,
            retain: false,
        }))
        .await
        .unwrap();

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
    broker_tx
        .send(BrokerMessage::Publish(PublishMessage {
            topic: "sensor/temperature".to_string(),
            payload: b"23.5".to_vec(),
            qos: 0,
            retain: true,
        }))
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Now connect and subscribe
    let (conn_tx, _conn_rx) = tokio::sync::oneshot::channel();
    broker_tx
        .send(BrokerMessage::Connect {
            client_id: "late_client".to_string(),
            sender: client_tx.clone(),
            clean_session: true,
            response: conn_tx,
            will_message: None,
        })
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(10)).await;

    broker_tx
        .send(BrokerMessage::Subscribe {
            client_id: "late_client".to_string(),
            topics: vec!["sensor/temperature".to_string()],
        })
        .await
        .unwrap();

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
    broker_tx
        .send(BrokerMessage::Publish(PublishMessage {
            topic: "sensor/temperature".to_string(),
            payload: b"23.5".to_vec(),
            qos: 0,
            retain: true,
        }))
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Clear retained message with empty payload
    broker_tx
        .send(BrokerMessage::Publish(PublishMessage {
            topic: "sensor/temperature".to_string(),
            payload: vec![],
            qos: 0,
            retain: true,
        }))
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Subscribe after clearing
    let (client_tx, mut client_rx) = mpsc::channel(100);
    let (conn_tx, _conn_rx) = tokio::sync::oneshot::channel();
    broker_tx
        .send(BrokerMessage::Connect {
            client_id: "test_client".to_string(),
            sender: client_tx.clone(),
            clean_session: true,
            response: conn_tx,
            will_message: None,
        })
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(10)).await;

    broker_tx
        .send(BrokerMessage::Subscribe {
            client_id: "test_client".to_string(),
            topics: vec!["sensor/temperature".to_string()],
        })
        .await
        .unwrap();

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
    let (conn_tx, _conn_rx) = tokio::sync::oneshot::channel();
    broker_tx
        .send(BrokerMessage::Connect {
            client_id: "test_client".to_string(),
            sender: client_tx.clone(),
            clean_session: true,
            response: conn_tx,
            will_message: None,
        })
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Subscribe to topic
    broker_tx
        .send(BrokerMessage::Subscribe {
            client_id: "test_client".to_string(),
            topics: vec!["qos/test".to_string()],
        })
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Publish QoS 1 message
    broker_tx
        .send(BrokerMessage::Publish(PublishMessage {
            topic: "qos/test".to_string(),
            payload: b"QoS 1 message".to_vec(),
            qos: 1,
            retain: false,
        }))
        .await
        .unwrap();

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
    let (conn_tx, _conn_rx) = tokio::sync::oneshot::channel();
    broker_tx
        .send(BrokerMessage::Connect {
            client_id: "test_client".to_string(),
            sender: client_tx.clone(),
            clean_session: true,
            response: conn_tx,
            will_message: None,
        })
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Subscribe to topic
    broker_tx
        .send(BrokerMessage::Subscribe {
            client_id: "test_client".to_string(),
            topics: vec!["qos2/test".to_string()],
        })
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Publish QoS 2 message
    broker_tx
        .send(BrokerMessage::Publish(PublishMessage {
            topic: "qos2/test".to_string(),
            payload: b"QoS 2 message".to_vec(),
            qos: 2,
            retain: false,
        }))
        .await
        .unwrap();

    // Should receive the message with QoS 2
    let msg = timeout(Duration::from_millis(100), client_rx.recv())
        .await
        .expect("Should receive message")
        .expect("Message should not be None");

    assert_eq!(msg.topic, "qos2/test");
    assert_eq!(msg.payload, b"QoS 2 message");
    assert_eq!(msg.qos, 2);
}

// Test persistent sessions
#[tokio::test]
async fn test_persistent_session() {
    let broker = Broker::new();
    let broker_tx = broker.get_sender();

    // Spawn broker task
    tokio::spawn(async move {
        broker.run().await;
    });

    // Connect client with clean_session=false (persistent)
    let (client_tx, _client_rx) = mpsc::channel(100);
    let (conn_tx, conn_rx) = tokio::sync::oneshot::channel();
    broker_tx
        .send(BrokerMessage::Connect {
            client_id: "persistent_client".to_string(),
            sender: client_tx.clone(),
            clean_session: false,
            response: conn_tx,
            will_message: None,
        })
        .await
        .unwrap();

    // First connection should have session_present=false
    let session_present = timeout(Duration::from_millis(100), conn_rx)
        .await
        .expect("Should receive session_present response")
        .expect("Channel should not be closed");
    assert!(!session_present, "First connection should not have session present");

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Subscribe to topic
    broker_tx
        .send(BrokerMessage::Subscribe {
            client_id: "persistent_client".to_string(),
            topics: vec!["persistent/test".to_string()],
        })
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Disconnect client
    broker_tx
        .send(BrokerMessage::Disconnect {
            client_id: "persistent_client".to_string(),
            graceful: true,
        })
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Publish QoS 1 message while client is offline
    broker_tx
        .send(BrokerMessage::Publish(PublishMessage {
            topic: "persistent/test".to_string(),
            payload: b"offline message".to_vec(),
            qos: 1,
            retain: false,
        }))
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Reconnect with same client_id and clean_session=false
    let (client_tx2, mut client_rx2) = mpsc::channel(100);
    let (conn_tx2, conn_rx2) = tokio::sync::oneshot::channel();
    broker_tx
        .send(BrokerMessage::Connect {
            client_id: "persistent_client".to_string(),
            sender: client_tx2.clone(),
            clean_session: false,
            response: conn_tx2,
            will_message: None,
        })
        .await
        .unwrap();

    // Reconnection should have session_present=true
    let session_present = timeout(Duration::from_millis(100), conn_rx2)
        .await
        .expect("Should receive session_present response")
        .expect("Channel should not be closed");
    assert!(session_present, "Reconnection should have session present");

    // Should receive the queued message
    let msg = timeout(Duration::from_millis(100), client_rx2.recv())
        .await
        .expect("Should receive queued message")
        .expect("Message should not be None");

    assert_eq!(msg.topic, "persistent/test");
    assert_eq!(msg.payload, b"offline message");
    assert_eq!(msg.qos, 1);

    // Publish new message after reconnect
    broker_tx
        .send(BrokerMessage::Publish(PublishMessage {
            topic: "persistent/test".to_string(),
            payload: b"online message".to_vec(),
            qos: 0,
            retain: false,
        }))
        .await
        .unwrap();

    // Should receive the new message (subscription was restored)
    let msg = timeout(Duration::from_millis(100), client_rx2.recv())
        .await
        .expect("Should receive new message")
        .expect("Message should not be None");

    assert_eq!(msg.topic, "persistent/test");
    assert_eq!(msg.payload, b"online message");
}

// Test clean session clears persistent session
#[tokio::test]
async fn test_clean_session_clears_persistent() {
    let broker = Broker::new();
    let broker_tx = broker.get_sender();

    // Spawn broker task
    tokio::spawn(async move {
        broker.run().await;
    });

    // Connect with persistent session
    let (client_tx, _client_rx) = mpsc::channel(100);
    let (conn_tx, _conn_rx) = tokio::sync::oneshot::channel();
    broker_tx
        .send(BrokerMessage::Connect {
            client_id: "test_clean".to_string(),
            sender: client_tx.clone(),
            clean_session: false,
            response: conn_tx,
            will_message: None,
        })
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Subscribe to topic
    broker_tx
        .send(BrokerMessage::Subscribe {
            client_id: "test_clean".to_string(),
            topics: vec!["test/topic".to_string()],
        })
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Disconnect
    broker_tx
        .send(BrokerMessage::Disconnect {
            client_id: "test_clean".to_string(),
            graceful: true,
        })
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Reconnect with clean_session=true (should clear session)
    let (client_tx2, mut client_rx2) = mpsc::channel(100);
    let (conn_tx2, conn_rx2) = tokio::sync::oneshot::channel();
    broker_tx
        .send(BrokerMessage::Connect {
            client_id: "test_clean".to_string(),
            sender: client_tx2.clone(),
            clean_session: true,
            response: conn_tx2,
            will_message: None,
        })
        .await
        .unwrap();

    // Should have session_present=false (session was cleared)
    let session_present = timeout(Duration::from_millis(100), conn_rx2)
        .await
        .expect("Should receive session_present response")
        .expect("Channel should not be closed");
    assert!(!session_present, "Session should be cleared with clean_session=true");

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Publish message - should not receive it because subscription was cleared
    broker_tx
        .send(BrokerMessage::Publish(PublishMessage {
            topic: "test/topic".to_string(),
            payload: b"test".to_vec(),
            qos: 0,
            retain: false,
        }))
        .await
        .unwrap();

    // Should NOT receive message (no subscription after clean session)
    let result = timeout(Duration::from_millis(100), client_rx2.recv()).await;
    assert!(result.is_err(), "Should not receive message without subscription");
}

#[tokio::test]
async fn test_will_message_on_abnormal_disconnect() {
    use mt_mqtt::broker::WillMessage;

    // Create broker
    let broker = Broker::new();
    let broker_tx = broker.get_sender();

    // Spawn broker task
    tokio::spawn(async move {
        broker.run().await;
    });

    // Connect subscriber to will topic
    let (sub_tx, mut sub_rx) = mpsc::channel(100);
    let (sub_conn_tx, _sub_conn_rx) = tokio::sync::oneshot::channel();
    broker_tx
        .send(BrokerMessage::Connect {
            client_id: "subscriber".to_string(),
            sender: sub_tx.clone(),
            clean_session: true,
            response: sub_conn_tx,
            will_message: None,
        })
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Subscribe to will topic
    broker_tx
        .send(BrokerMessage::Subscribe {
            client_id: "subscriber".to_string(),
            topics: vec!["status/client".to_string()],
        })
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Connect client with will message
    let (client_tx, _client_rx) = mpsc::channel(100);
    let (conn_tx, _conn_rx) = tokio::sync::oneshot::channel();
    broker_tx
        .send(BrokerMessage::Connect {
            client_id: "will_client".to_string(),
            sender: client_tx.clone(),
            clean_session: true,
            response: conn_tx,
            will_message: Some(WillMessage {
                topic: "status/client".to_string(),
                payload: b"offline".to_vec(),
                qos: 0,
                retain: false,
            }),
        })
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Disconnect abnormally (graceful=false)
    broker_tx
        .send(BrokerMessage::Disconnect {
            client_id: "will_client".to_string(),
            graceful: false,
        })
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Subscriber should receive will message
    let msg = timeout(Duration::from_millis(100), sub_rx.recv())
        .await
        .expect("Should receive will message")
        .expect("Channel should not be closed");

    assert_eq!(msg.topic, "status/client");
    assert_eq!(msg.payload, b"offline");
}

#[tokio::test]
async fn test_will_message_not_sent_on_graceful_disconnect() {
    use mt_mqtt::broker::WillMessage;

    // Create broker
    let broker = Broker::new();
    let broker_tx = broker.get_sender();

    // Spawn broker task
    tokio::spawn(async move {
        broker.run().await;
    });

    // Connect subscriber to will topic
    let (sub_tx, mut sub_rx) = mpsc::channel(100);
    let (sub_conn_tx, _sub_conn_rx) = tokio::sync::oneshot::channel();
    broker_tx
        .send(BrokerMessage::Connect {
            client_id: "subscriber".to_string(),
            sender: sub_tx.clone(),
            clean_session: true,
            response: sub_conn_tx,
            will_message: None,
        })
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Subscribe to will topic
    broker_tx
        .send(BrokerMessage::Subscribe {
            client_id: "subscriber".to_string(),
            topics: vec!["status/client".to_string()],
        })
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Connect client with will message
    let (client_tx, _client_rx) = mpsc::channel(100);
    let (conn_tx, _conn_rx) = tokio::sync::oneshot::channel();
    broker_tx
        .send(BrokerMessage::Connect {
            client_id: "will_client".to_string(),
            sender: client_tx.clone(),
            clean_session: true,
            response: conn_tx,
            will_message: Some(WillMessage {
                topic: "status/client".to_string(),
                payload: b"offline".to_vec(),
                qos: 0,
                retain: false,
            }),
        })
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Disconnect gracefully (graceful=true)
    broker_tx
        .send(BrokerMessage::Disconnect {
            client_id: "will_client".to_string(),
            graceful: true,
        })
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Subscriber should NOT receive will message
    let result = timeout(Duration::from_millis(100), sub_rx.recv()).await;
    assert!(result.is_err(), "Should not receive will message on graceful disconnect");
}

#[tokio::test]
async fn test_will_message_with_retain() {
    use mt_mqtt::broker::WillMessage;

    // Create broker
    let broker = Broker::new();
    let broker_tx = broker.get_sender();

    // Spawn broker task
    tokio::spawn(async move {
        broker.run().await;
    });

    // Connect client with retained will message
    let (client_tx, _client_rx) = mpsc::channel(100);
    let (conn_tx, _conn_rx) = tokio::sync::oneshot::channel();
    broker_tx
        .send(BrokerMessage::Connect {
            client_id: "will_client".to_string(),
            sender: client_tx.clone(),
            clean_session: true,
            response: conn_tx,
            will_message: Some(WillMessage {
                topic: "status/device".to_string(),
                payload: b"disconnected".to_vec(),
                qos: 0,
                retain: true,
            }),
        })
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Disconnect abnormally to trigger will message
    broker_tx
        .send(BrokerMessage::Disconnect {
            client_id: "will_client".to_string(),
            graceful: false,
        })
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Now connect a new subscriber
    let (sub_tx, mut sub_rx) = mpsc::channel(100);
    let (sub_conn_tx, _sub_conn_rx) = tokio::sync::oneshot::channel();
    broker_tx
        .send(BrokerMessage::Connect {
            client_id: "late_subscriber".to_string(),
            sender: sub_tx.clone(),
            clean_session: true,
            response: sub_conn_tx,
            will_message: None,
        })
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Subscribe to the topic (should receive retained will message)
    broker_tx
        .send(BrokerMessage::Subscribe {
            client_id: "late_subscriber".to_string(),
            topics: vec!["status/device".to_string()],
        })
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Should receive retained will message
    let msg = timeout(Duration::from_millis(100), sub_rx.recv())
        .await
        .expect("Should receive retained will message")
        .expect("Channel should not be closed");

    assert_eq!(msg.topic, "status/device");
    assert_eq!(msg.payload, b"disconnected");
    assert!(msg.retain, "Message should have retain flag set");
}
