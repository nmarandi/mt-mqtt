# mt-mqtt

A multi-threaded MQTT broker implementation in Rust using Tokio for asynchronous I/O.

## Status

🚧 **Under Active Development** - This project is in early stages and not yet production-ready.

## Features

- Asynchronous MQTT broker built with Tokio
- TCP connection support (ports 1883 and 8883)
- Frame-based packet encoding/decoding
- Topic subscription management with wildcard support (`+` and `#`)
- Publisher/Subscriber pattern implementation
- MQTT v3.1.1 protocol support

## Architecture

- **Broker**: Central message routing and client management
- **Server**: TCP listener and connection handling
- **Client**: Individual client connection management
- **Frame**: MQTT packet serialization/deserialization
- **Topic**: Topic tree for subscription matching
- **Publisher/Subscriber**: Message distribution logic

## Usage

```rust
use mt_mqtt::start_broker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    start_broker().await
}
```

## Building

```bash
cargo build --release
```

## Testing

```bash
cargo test
```

## Dependencies

- **tokio**: Asynchronous runtime
- **bytes**: Byte buffer utilities
- **num-traits/num-derive**: Numeric type traits
- **strum**: Enum utilities

## Roadmap

- [ ] Complete MQTT 3.1.1 protocol implementation
- [ ] Add TLS/SSL support for secure connections
- [ ] Implement QoS levels (0, 1, 2)
- [ ] Add persistence layer
- [ ] WebSocket support
- [ ] MQTT 5.0 support
- [ ] Clustering and scalability features

## License

See LICENSE file for details.
