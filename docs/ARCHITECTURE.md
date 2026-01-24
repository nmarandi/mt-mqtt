# mt-mqtt Architecture

## Project Structure

```
mt-mqtt/
├── src/
│   ├── lib.rs              # Library entry point
│   ├── broker/             # Broker module - message routing & client management
│   │   ├── mod.rs         # Main broker implementation
│   │   ├── publisher.rs   # Publisher logic
│   │   └── subscriber.rs  # Subscriber logic
│   ├── client.rs          # Individual client connection handling
│   ├── server.rs          # TCP server & connection listener
│   ├── protocol/          # MQTT protocol implementation
│   │   ├── mod.rs        # Protocol module exports
│   │   ├── definitions.rs # MQTT data structures and enums
│   │   ├── packet.rs     # MQTT packet definitions
│   │   └── frame/        # Frame encoding/decoding
│   │       ├── mod.rs    # Frame struct and serialization logic
│   │       ├── encoder.rs # Packet encoding functions
│   │       └── decoder.rs # Packet decoding functions
│   └── topic.rs          # Topic tree for subscription matching
├── Cargo.toml            # Project dependencies
└── README.md             # Project documentation
```

## Module Responsibilities

### `lib.rs`
- Entry point for the library
- Exports public API (`start_broker()`)
- Module declarations and organization

### `broker/`
Central message broker implementation that manages the flow of messages between publishers and subscribers.

- **mod.rs**: Core broker logic with message routing
- **publisher.rs**: Publisher-specific functionality
- **subscriber.rs**: Subscriber-specific functionality

### `client.rs`
Handles individual client connections:
- Frame reading/writing over TCP
- Buffer management
- Client state and identification
- Protocol message handling loop

### `server.rs`
TCP server implementation:
- Listens on ports 1883 (unsecure) and 8883 (secure)
- Accepts incoming connections
- Spawns client handlers asynchronously

### `protocol/`
Complete MQTT protocol implementation organized by concern:

#### `definitions.rs`
- MQTT control packet types
- Flags and headers
- Reason codes
- Property definitions
- Connect/Subscribe/Publish structures

#### `packet.rs`
- Control packet structures for all MQTT packet types
- Variable headers
- Payloads
- Helper functions

#### `frame/`
Frame encoding and decoding logic:

- **mod.rs**: Main `Frame` struct with serialize/deserialize methods
- **encoder.rs**: Functions to encode MQTT packets to bytes
- **decoder.rs**: Functions to decode bytes into MQTT packets

### `topic.rs`
Topic subscription tree implementation:
- Hierarchical topic matching
- Wildcard support (`+` single-level, `#` multi-level)
- Subscriber management per topic

## Data Flow

```
TCP Client
    ↓
Server (listener)
    ↓
Client (spawned)
    ↓
Frame (deserialize)
    ↓
Broker (message routing)
    ↓
Topic Tree (subscription matching)
    ↓
Client (write to subscribers)
    ↓
Frame (serialize)
    ↓
TCP Client
```

## Key Design Patterns

### Async/Await with Tokio
- All I/O operations are asynchronous
- Each client runs in its own Tokio task
- Non-blocking message processing

### Message Passing
- Broker uses channels for inter-task communication
- `BrokerMessage` enum for command pattern
- Decoupled publisher/subscriber handling

### Modular Protocol Implementation
- Protocol logic separated from business logic
- Frame encoding/decoding isolated in dedicated modules
- Easy to test and maintain

## Adding New Features

### Adding a new MQTT packet type:
1. Add packet type enum to `protocol/definitions.rs`
2. Define packet structure in `protocol/packet.rs`
3. Add encoder function in `protocol/frame/encoder.rs`
4. Add decoder function in `protocol/frame/decoder.rs`
5. Update `Frame::deserialize()` and `Frame::serialize()` in `protocol/frame/mod.rs`
6. Handle packet in `client.rs` run loop

### Adding broker functionality:
1. Define message type in `broker/mod.rs::BrokerMessage`
2. Add handler in `broker/mod.rs::Broker::run()`
3. Send messages from client handler

### Extending topic features:
1. Modify `topic.rs::TopicTree` structure
2. Update subscribe/unsubscribe logic
3. Adjust message routing in broker
