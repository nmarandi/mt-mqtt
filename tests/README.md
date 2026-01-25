# MT-MQTT Test Suite

This directory contains integration tests for the MT-MQTT broker using mosquitto_pub and mosquitto_sub clients.

## Running Tests

### Run All Tests
```bash
# Using default MQTT 3.1.1
./run_all_tests.sh

# Using MQTT 5.0
./run_all_tests.sh 5
```

### Run Individual Tests
```bash
# Using default MQTT 3.1.1
./test_qos_levels.sh

# Using MQTT 5.0
./test_qos_levels.sh 5
```

## Test Scripts

- **test_basic_pubsub.sh** - Basic publish/subscribe functionality
- **test_wildcard_subscriptions.sh** - Tests `+` and `#` wildcards
- **test_multiple_subscribers.sh** - Multiple subscribers to same topic
- **test_qos_levels.sh** - QoS 0, 1, and 2 message delivery
- **test_retained_messages.sh** - Retained message functionality
- **test_high_volume.sh** - High volume message throughput

## Protocol Versions

The tests support both MQTT 3.1.1 and MQTT 5.0:
- **311** - MQTT 3.1.1 (default)
- **5** - MQTT 5.0

Example:
```bash
./test_qos_levels.sh 311  # MQTT 3.1.1
./test_qos_levels.sh 5    # MQTT 5.0
```

## Requirements

- Mosquitto client tools (`mosquitto_pub` and `mosquitto_sub`)
- MT-MQTT broker running on localhost:1883

## Notes

- The broker currently supports MQTT 3.1.1 format packets
- Tests use temporary files in `/tmp/` for output verification
- Each test cleans up its temporary files after completion
