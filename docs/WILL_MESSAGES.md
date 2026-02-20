# Will Messages (Last Will and Testament)

## Overview

Will Messages, also known as Last Will and Testament (LWT), are a key feature of MQTT that enables automatic notification when a client disconnects unexpectedly. This is particularly useful for device presence detection and status monitoring.

## How It Works

When a client connects to the broker, it can specify a will message in the CONNECT packet. The broker stores this message and will publish it automatically if the client disconnects abnormally.

### When Will Messages Are Published

✅ **Published on:**
- Network failure or connection loss
- Client crash or forced termination
- Client timeout (keep-alive failure)
- Any abnormal disconnect

❌ **NOT published on:**
- Graceful disconnect (client sends DISCONNECT packet)
- Normal shutdown

## Implementation Details

### Architecture

1. **Client Side** (`src/client.rs`)
   - Extracts will message from CONNECT packet
   - Sends will message info to broker during connection
   - Tracks graceful vs abnormal disconnects

2. **Broker Side** (`src/broker/mod.rs`)
   - Stores will messages per client ID
   - Publishes will message on abnormal disconnect
   - Removes will message on graceful disconnect (without publishing)
   - Supports retained will messages

### Data Structures

```rust
pub struct WillMessage {
    pub topic: String,
    pub payload: Vec<u8>,
    pub qos: u8,
    pub retain: bool,
}
```

## Usage Examples

### Using Mosquitto Client

```bash
# Connect with a will message
mosquitto_pub -h localhost -p 1883 -t "data/sensor" -m "online" \
  --will-topic "status/sensor" \
  --will-payload "offline" \
  --will-qos 1 \
  --will-retain

# Subscribe to see will messages
mosquitto_sub -h localhost -p 1883 -t "status/#" -v
```

### Programmatic Usage

```rust
use mt_mqtt::broker::{BrokerMessage, WillMessage};

// Connect with will message
let will_message = Some(WillMessage {
    topic: "status/device".to_string(),
    payload: b"offline".to_vec(),
    qos: 1,
    retain: true,
});

broker_tx.send(BrokerMessage::Connect {
    client_id: "device123".to_string(),
    clean_session: true,
    sender: msg_sender,
    response: response_tx,
    will_message,
}).await?;
```

## Common Use Cases

### 1. Device Presence Detection

```bash
# Device publishes "online" on connect with will message "offline"
mosquitto_pub -h localhost -t "device/123/status" -m "online" \
  --will-topic "device/123/status" --will-payload "offline" --will-retain
```

Subscribers to `device/123/status` will automatically know when the device goes offline.

### 2. Service Health Monitoring

```rust
// Service connects with will message indicating service failure
WillMessage {
    topic: "services/api/health".to_string(),
    payload: b"DOWN".to_vec(),
    qos: 1,
    retain: true,
}
```

### 3. Last Known State

Use retained will messages to preserve the last known state:

```bash
mosquitto_pub -h localhost -t "sensor/temp" -m "23.5" \
  --will-topic "sensor/temp/status" \
  --will-payload "SENSOR_FAILURE" \
  --will-qos 1 --will-retain
```

## QoS Support

Will messages support all three QoS levels:

- **QoS 0**: At-most-once delivery (fire and forget)
- **QoS 1**: At-least-once delivery with acknowledgment
- **QoS 2**: Exactly-once delivery with 4-way handshake

```bash
# QoS 0 will message
--will-qos 0

# QoS 1 will message (recommended)
--will-qos 1

# QoS 2 will message
--will-qos 2
```

## Retained Will Messages

Will messages can be retained, making them available to late-joining subscribers:

```bash
mosquitto_pub -h localhost -t "device/sensor" -m "data" \
  --will-topic "device/sensor/status" \
  --will-payload "offline" \
  --will-retain
```

When the device disconnects abnormally, the retained will message ensures that any future subscriber immediately knows the device is offline.

## Testing

The implementation includes comprehensive tests:

### Integration Tests

1. **test_will_message_on_abnormal_disconnect**
   - Verifies will message is published when client disconnects abnormally
   - Tests message routing to subscribers

2. **test_will_message_not_sent_on_graceful_disconnect**
   - Verifies will message is NOT published on graceful DISCONNECT
   - Ensures proper cleanup

3. **test_will_message_with_retain**
   - Verifies retained will messages work correctly
   - Tests late-joining subscribers receive retained will message

Run tests:
```bash
cargo test will_message
```

## Protocol Compliance

This implementation follows the MQTT 3.1.1 specification:

- ✅ Will message fields parsed from CONNECT packet
- ✅ Will message stored until disconnect
- ✅ Published only on abnormal disconnect
- ✅ Supports QoS 0, 1, 2
- ✅ Supports retain flag
- ✅ Graceful disconnect suppresses publication

## Best Practices

1. **Always use retain flag for status messages**
   ```bash
   --will-retain
   ```
   This ensures late-joining subscribers immediately know the device status.

2. **Use QoS 1 or 2 for critical will messages**
   ```bash
   --will-qos 1
   ```
   Ensures delivery even during network issues.

3. **Keep will messages small**
   - Use concise payloads like "offline", "DOWN", "FAILED"
   - Reduces bandwidth and storage

4. **Use descriptive topics**
   ```
   Good: device/sensor123/status
   Bad:  sensor/status
   ```

5. **Publish "online" immediately after connect**
   ```bash
   mosquitto_pub -t "device/status" -m "online" \
     --will-topic "device/status" --will-payload "offline"
   ```

## Troubleshooting

### Will message not published

Check:
- Is the disconnect abnormal? (Graceful DISCONNECT suppresses will message)
- Is the broker receiving the will message in CONNECT?
- Are there subscribers to the will topic?

### Will message published unexpectedly

Check:
- Is the client sending DISCONNECT before closing?
- Is the keep-alive timeout too short?
- Are there network issues causing timeouts?

## Performance Considerations

- Will messages are stored in memory per client
- No performance impact on normal message routing
- Published using standard message distribution path
- Retained will messages use same storage as regular retained messages

## Future Enhancements

Planned improvements:
- [ ] Persistence of will messages across broker restarts
- [ ] Will message delays (MQTT 5.0 feature)
- [ ] Will message expiry (MQTT 5.0 feature)
