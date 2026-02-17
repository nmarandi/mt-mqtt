# MT-MQTT GitHub Copilot Instructions

This repository contains MT-MQTT, a high-performance async MQTT broker implementation in Rust built with Tokio. These instructions help GitHub Copilot understand the project structure, coding standards, and best practices.

## Project Overview

MT-MQTT is an MQTT 3.1.1 compliant broker with support for:
- QoS 0, 1, and 2 message delivery
- Wildcard subscriptions (`+` single-level, `#` multi-level)
- Retained messages
- Persistent sessions (in development)
- Asynchronous I/O using Tokio

**Status:** Core MQTT functionality fully implemented and tested (39/39 tests passing)

## Technology Stack

### Core Dependencies
- **Rust Edition:** 2021 (minimum Rust 1.70+)
- **Runtime:** Tokio 1.35+ with full features
- **Bytes:** bytes 1.5+ for zero-copy buffer operations
- **Numeric:** num-traits 0.2+, num-derive 0.4+
- **Enums:** strum 0.25+ for enum serialization
- **Logging:** tracing 0.1+ with tracing-subscriber

### Development Dependencies
- **Testing:** tokio-test 0.4+
- **Benchmarking:** criterion 0.5+

## Project Structure

```
mt-mqtt/
├── src/
│   ├── lib.rs              # Library entry point, exports start_broker()
│   ├── main.rs             # Binary entry point
│   ├── broker/             # Central message routing & client management
│   │   ├── mod.rs          # Core broker with BrokerMessage enum
│   │   ├── publisher.rs    # Publisher logic
│   │   └── subscriber.rs   # Subscriber logic
│   ├── client.rs           # Per-connection handler, MQTT protocol state
│   ├── server.rs           # TCP listener on port 1883
│   ├── protocol/           # MQTT protocol implementation
│   │   ├── mod.rs          # Protocol module exports
│   │   ├── definitions.rs  # MQTT data types (PacketType, QoS, etc.)
│   │   ├── packet.rs       # MQTT packet structures
│   │   └── frame/          # Packet serialization/deserialization
│   │       ├── mod.rs      # Frame struct with serialize/deserialize
│   │       ├── encoder.rs  # Packet to bytes encoding
│   │       └── decoder.rs  # Bytes to packet decoding
│   ├── topic.rs            # Trie-based topic tree with wildcard support
│   ├── packet_id.rs        # MQTT packet identifier management
│   └── message_state.rs    # In-flight message tracking for QoS 1/2
├── tests/
│   ├── integration_tests.rs # Integration tests (8 tests)
│   └── README.md            # E2E testing instructions
├── benches/
│   └── broker_bench.rs      # Performance benchmarks
├── docs/                    # Architecture and implementation documentation
└── Cargo.toml
```

## Build, Test, and Development Commands

### Building
```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run the broker
cargo run --release --bin mt-mqtt
```

### Testing
```bash
# Run all tests (unit + integration)
cargo test

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test integration_tests

# Run tests with output
cargo test -- --nocapture

# Run a specific test
cargo test test_name
```

**Test Coverage:**
- Unit Tests: 31/31 passing (TopicTree, Protocol, PacketID, MessageState)
- Integration Tests: 8/8 passing (Broker routing, Wildcards, QoS levels)

### Linting and Formatting
```bash
# Format code (REQUIRED before committing)
cargo fmt

# Check formatting without applying changes
cargo fmt -- --check

# Run Clippy linter (REQUIRED, must pass)
cargo clippy

# Run Clippy with all warnings as errors
cargo clippy -- -D warnings
```

**rustfmt.toml settings:**
- `max_width = 150` - Line length limit
- `merge_imports = true` - Consolidate imports (unstable, requires nightly)
- `reorder_impl_items = true` - Sort impl block items (unstable, requires nightly)
- `empty_item_single_line = false` - Keep empty items on multiple lines (unstable, requires nightly)

**Note:** Some rustfmt settings are unstable and only work with nightly Rust. On stable, cargo fmt will use default settings and may show warnings.

### Benchmarking
```bash
# Run performance benchmarks
cargo bench
```

## Coding Standards and Best Practices

### General Guidelines
1. **Follow Rust idioms:** Use idiomatic Rust patterns and conventions
2. **Use cargo fmt:** Always format code before committing
3. **Pass cargo clippy:** Fix all Clippy warnings
4. **Write tests:** Add tests for new features (unit + integration)
5. **Update docs:** Keep documentation current with code changes
6. **Async-first:** All I/O operations must be async using Tokio

### Code Style
- Use descriptive variable names (e.g., `client_id`, `topic_name`)
- Prefer explicit types when it improves clarity
- Use `Result<T, E>` for error handling, avoid `.unwrap()` in production code
- Add doc comments (`///`) for public APIs
- Keep functions focused and single-purpose
- Maximum line length: 150 characters (per rustfmt.toml)

### Naming Conventions
- **Structs/Enums:** PascalCase (e.g., `BrokerMessage`, `TopicTree`)
- **Functions/Variables:** snake_case (e.g., `handle_publish`, `client_id`)
- **Constants:** SCREAMING_SNAKE_CASE (e.g., `MAX_CLIENTS`)
- **Module files:** snake_case (e.g., `message_state.rs`)

### MQTT Protocol Specifics
- **Packet IDs:** Use `PacketIdManager` for allocation and tracking
- **QoS levels:** Handle QoS 0 (fire-and-forget), QoS 1 (at-least-once), QoS 2 (exactly-once)
- **Topic matching:** Use `TopicTree` for efficient wildcard matching
- **Message state:** Track in-flight messages using `MessageState`
- **Session persistence:** Use `Session` struct for clean_session=false

### Error Handling
```rust
// Use Result types
pub async fn handle_packet(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    // ... implementation
}

// Propagate errors with ?
let packet = self.read_packet().await?;

// Use expect() only for initialization/setup
let addr = "0.0.0.0:1883".parse().expect("Valid address");
```

### Async Patterns
```rust
// Use tokio::spawn for concurrent tasks
tokio::spawn(async move {
    client.run().await;
});

// Use channels for message passing
let (tx, mut rx) = mpsc::channel::<BrokerMessage>(100);

// Use select! for multiple async operations
tokio::select! {
    result = client.read() => { /* ... */ }
    msg = rx.recv() => { /* ... */ }
}
```

## Architecture Patterns

### Message Passing (Broker)
- Broker uses `mpsc::channel` for inter-task communication
- `BrokerMessage` enum defines command pattern
- Clients send commands to broker via channels
- Broker routes messages to appropriate subscribers

### Data Flow
```
TCP Client → Server → Client Handler → Frame Deserialize
    → Broker (routing) → Topic Tree (matching)
    → Client Handler → Frame Serialize → TCP Client
```

### Adding New MQTT Packet Types
1. Add packet type to `protocol/definitions.rs` (PacketType enum)
2. Define packet structure in `protocol/packet.rs`
3. Add encoder function in `protocol/frame/encoder.rs`
4. Add decoder function in `protocol/frame/decoder.rs`
5. Update `Frame::deserialize()` and `Frame::serialize()` in `protocol/frame/mod.rs`
6. Handle packet in `client.rs` run loop
7. Add tests in relevant test modules

### Adding Broker Functionality
1. Define message type in `broker/mod.rs::BrokerMessage` enum
2. Add handler in `broker/mod.rs::Broker::run()` match statement
3. Send messages from client handler using broker channel
4. Add unit tests for the new functionality
5. Add integration test if it affects client behavior

## Testing Guidelines

### Unit Tests
- Place tests in the same file using `#[cfg(test)]` module
- Test individual functions and components in isolation
- Use `tokio-test` for async test utilities
- Example locations: `src/topic.rs`, `src/protocol/frame/tests.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topic_matching() {
        let tree = TopicTree::new();
        // ... test implementation
    }

    #[tokio::test]
    async fn test_async_function() {
        // ... async test implementation
    }
}
```

### Integration Tests
- Located in `tests/integration_tests.rs`
- Test full broker behavior with real client connections
- Use `tokio::test` for async integration tests
- Test complete workflows (connect, subscribe, publish, disconnect)

### E2E Tests
- Located in `tests/` directory with shell scripts
- Use Mosquitto client tools (`mosquitto_pub`, `mosquitto_sub`)
- Requires running broker instance
- Test with real MQTT clients

## Documentation Standards

### Code Comments
- Use `///` for public API documentation (shows in cargo doc)
- Use `//` for implementation details
- Document non-obvious behavior, edge cases, MQTT spec requirements
- Keep comments up-to-date with code changes

### README and Docs
- Update README.md for user-facing changes
- Update docs/ for architectural changes
- Document new features in README's "Features" section
- Update test counts when adding/modifying tests

## Files and Directories to Avoid Modifying

**Do not modify unless explicitly required:**
- `.git/` - Git repository internals
- `target/` - Build artifacts (auto-generated)
- `Cargo.lock` - Locked dependency versions (committed but managed by Cargo)

**Be cautious with:**
- `LICENSE` - MIT license, only change if legally necessary
- `Cargo.toml` - Only add dependencies if truly needed
- `rustfmt.toml` - Established formatting rules

## Common Tasks

### Adding a New Feature
1. Create an issue describing the feature
2. Write failing tests first (TDD approach)
3. Implement the feature
4. Ensure all tests pass (`cargo test`)
5. Run formatting (`cargo fmt`) and linting (`cargo clippy`)
6. Update documentation (README, code comments)
7. Add integration test if needed
8. Create a pull request

### Fixing a Bug
1. Write a test that reproduces the bug
2. Verify the test fails
3. Fix the bug
4. Verify the test now passes
5. Ensure no regressions (`cargo test`)
6. Update documentation if needed

### Optimizing Performance
1. Identify bottleneck with benchmarks (`cargo bench`)
2. Make targeted changes
3. Re-run benchmarks to verify improvement
4. Ensure tests still pass
5. Document performance characteristics

## MQTT 3.1.1 Specification Compliance

When implementing MQTT features, always refer to the [MQTT 3.1.1 OASIS specification](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/mqtt-v3.1.1.html).

**Key Requirements:**
- CONNECT must be first packet from client
- CONNACK must include session_present flag
- QoS 0: Fire and forget, no acknowledgment
- QoS 1: At-least-once, requires PUBACK
- QoS 2: Exactly-once, requires PUBREC → PUBREL → PUBCOMP
- Wildcard '#' must be last character in topic filter
- Wildcard '+' matches single topic level
- Packet identifiers must be non-zero and unique per client
- Keep-alive with PINGREQ/PINGRESP

## Security Considerations

**Current Status:** No authentication or TLS implemented yet

**Future Security Features:**
- TLS/SSL support (port 8883)
- Username/password authentication
- Access Control Lists (ACL)
- Rate limiting
- Input validation (currently basic)

**When adding security features:**
1. Never log sensitive data (passwords, tokens)
2. Use constant-time comparison for secrets
3. Validate all client input
4. Follow OWASP guidelines for auth/authz
5. Add security tests

## Performance Expectations

**Current Benchmarks:**
- Message routing: ~50k msg/sec
- Topic matching (exact): ~500k ops/sec
- Topic matching (wildcard): ~200k ops/sec
- Subscribe operation: ~100k ops/sec

**Performance Guidelines:**
- Minimize allocations in hot paths
- Use `bytes::Bytes` for zero-copy operations
- Avoid blocking operations in async code
- Use `tokio::spawn` for CPU-intensive work
- Profile before optimizing (use `cargo flamegraph` or criterion)

## Examples

### Subscribe to Topic
```rust
// Client subscribes to a topic
let subscribe = Subscribe {
    packet_id: 1,
    topics: vec![("sensor/+/temperature".to_string(), QoS::AtLeastOnce)],
};
```

### Publish Message with QoS 1
```rust
// Publish with QoS 1 requires acknowledgment
let publish = Publish {
    packet_id: Some(42),
    topic: "sensor/temp".to_string(),
    payload: Bytes::from("23.5"),
    qos: QoS::AtLeastOnce,
    retain: false,
    dup: false,
};
```

### Topic Matching with Wildcards
```rust
// Single-level wildcard
"sensor/+/temperature" matches "sensor/living_room/temperature"

// Multi-level wildcard
"sensor/#" matches "sensor/living_room/temperature" and "sensor/bedroom/humidity"
```

## Resources

- **MQTT Spec:** https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/mqtt-v3.1.1.html
- **Tokio Docs:** https://tokio.rs/
- **Testing Guide:** https://mosquitto.org/ (Mosquitto client tools)
- **Architecture:** See `docs/ARCHITECTURE.md`
- **Implementation Plan:** See `docs/IMPLEMENTATION_PLAN.md`

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes following these guidelines
4. Run tests, fmt, and clippy
5. Submit a pull request with clear description

---

**Note:** This project is under active development. Core MQTT 3.1.1 functionality is stable, but authentication, persistence, and TLS are planned for future releases.
