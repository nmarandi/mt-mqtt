# MT-MQTT Testing Guide

## Overview

This project includes comprehensive testing at multiple levels:

1. **Unit Tests** - Test individual components in isolation
2. **Integration Tests** - Test component interactions
3. **End-to-End Tests** - Test complete scenarios with real MQTT clients
4. **Benchmarks** - Performance testing

## Running Tests

### All Tests (Recommended)

**Linux/macOS:**
```bash
./run_tests.sh
```

**Windows:**
```cmd
run_tests.bat
```

### Individual Test Suites

**Unit Tests Only:**
```bash
cargo test --lib
```

**Integration Tests Only:**
```bash
cargo test --test integration_tests
```

**E2E Tests (requires running broker):**
```bash
bash tests/run_all_tests.sh
```

**Benchmarks:**
```bash
cargo bench
```

## Test Coverage

### Unit Tests

Located in `src/*/tests.rs` files:

- **Topic Tree Tests** (`src/topic/tests.rs`)
  - Exact topic matching
  - Single-level wildcard (+)
  - Multi-level wildcard (#)
  - Multiple subscribers
  - Subscription/unsubscription
  - Case sensitivity
  - Overlapping subscriptions

- **Frame Codec Tests** (`src/protocol/frame/tests.rs`)
  - MQTT 3.1.1 packet deserialization
  - Packet serialization (CONNACK, etc.)
  - Incomplete packet handling
  - Variable byte integer encoding/decoding
  - String encoding/decoding with bounds checking

### Integration Tests

Located in `tests/integration_tests.rs`:

- Broker message routing
- Wildcard subscription routing
- Multiple subscriber delivery
- Client connection/disconnection
- Subscription management

### E2E Tests

Located in `tests/test_*.sh`:

1. **Basic Pub/Sub** - Simple publish and subscribe
2. **Wildcard Subscriptions** - Test # and + wildcards
3. **Multiple Subscribers** - Message fanout to multiple clients
4. **Retained Messages** - Late subscriber receives retained messages
5. **High Volume** - 100 messages throughput test

## Test Scenarios

### Scenario 1: Basic Communication
```bash
bash tests/test_basic_pubsub.sh
```
Tests basic MQTT publish/subscribe functionality.

### Scenario 2: Wildcard Patterns
```bash
bash tests/test_wildcard_subscriptions.sh
```
Validates MQTT wildcard subscription matching.

### Scenario 3: Broadcast
```bash
bash tests/test_multiple_subscribers.sh
```
Ensures messages reach all subscribers.

### Scenario 4: Retained Messages
```bash
bash tests/test_retained_messages.sh
```
Tests retained message delivery to late joiners.

### Scenario 5: Load Testing
```bash
bash tests/test_high_volume.sh
```
Sends 100 messages to test throughput.

## Manual Testing

### Using Mosquitto Clients

**Subscribe:**
```bash
mosquitto_sub -h localhost -p 1883 -t "test/topic" -v
```

**Publish:**
```bash
mosquitto_pub -h localhost -p 1883 -t "test/topic" -m "Hello MQTT"
```

**Wildcard Subscribe:**
```bash
# Multi-level wildcard
mosquitto_sub -h localhost -p 1883 -t "sensor/#" -v

# Single-level wildcard
mosquitto_sub -h localhost -p 1883 -t "device/+/status" -v
```

**Retained Message:**
```bash
mosquitto_pub -h localhost -p 1883 -t "status" -m "online" -r
```

## Debugging Tests

### Enable Verbose Output

Add this to tests to see detailed logs:
```rust
env_logger::init();
```

### Check Broker Logs

When using the test runner, broker output is saved to:
- Linux/macOS: `/tmp/broker_output.log`
- Windows: `broker_output.log`

### Run Single Test

```bash
cargo test test_name -- --nocapture
```

## Continuous Integration

For CI/CD pipelines:

```bash
# Install dependencies
cargo build --release

# Run all tests
cargo test --all
cargo test --test integration_tests

# Run benchmarks (optional)
cargo bench --no-fail-fast
```

## Performance Testing

### Run Benchmarks
```bash
cargo bench
```

Benchmark results include:
- Topic matching performance (exact, single-level, multi-level wildcards)
- Subscription operations (subscribe/unsubscribe)
- Scalability with many subscribers

### View Results
Results are saved to `target/criterion/` with HTML reports.

## Test-Driven Development

When adding new features:

1. Write failing test first
2. Implement minimum code to pass
3. Refactor while keeping tests green

Example:
```rust
#[test]
fn test_new_feature() {
    // Arrange
    let mut broker = Broker::new();
    
    // Act
    let result = broker.new_feature();
    
    // Assert
    assert!(result.is_ok());
}
```

## Common Issues

### "Broker not running"
Start the broker first:
```bash
cargo run --bin mt-mqtt
```

### "Port 1883 already in use"
Kill existing broker:
```bash
# Linux/macOS
pkill mt-mqtt

# Windows
taskkill /F /IM mt-mqtt.exe
```

### Tests timeout
Increase timeout in test:
```rust
tokio::time::timeout(
    tokio::time::Duration::from_secs(5),
    test_future
).await
```

## Contributing Tests

When adding tests:

1. Use descriptive test names: `test_feature_scenario_expected`
2. Follow AAA pattern: Arrange, Act, Assert
3. Keep tests isolated (no shared state)
4. Add both positive and negative test cases
5. Document complex test scenarios

Example:
```rust
#[test]
fn test_subscribe_duplicate_topic_adds_subscriber() {
    // Arrange
    let mut tree = TopicTree::new();
    tree.subscribe("test/topic", "client1");
    
    // Act
    tree.subscribe("test/topic", "client2");
    
    // Assert
    let subscribers = tree.get_subscribers("test/topic");
    assert_eq!(subscribers.len(), 2);
}
```
