# Disk Persistence Architecture

## Overview

MT-MQTT supports optional disk persistence for sessions and messages through a flexible, trait-based architecture. This allows the broker to survive restarts without losing session state or queued messages.

## Design Philosophy

### Trait-Based Abstraction

The persistence layer is built around the `PersistenceBackend` trait, which defines a common interface for all storage backends:

```rust
#[async_trait]
pub trait PersistenceBackend: Send + Sync {
    async fn save_session(&self, session: &PersistedSession) -> Result<(), PersistenceError>;
    async fn load_session(&self, client_id: &str) -> Result<Option<PersistedSession>, PersistenceError>;
    // ... more methods
}
```

This design makes it easy to:
- **Swap backends**: Replace SQLite with RocksDB, PostgreSQL, etc. without changing broker code
- **Test easily**: Use `InMemoryBackend` for tests, SQLite for production
- **Add features**: New backends can be added without modifying existing code

## Supported Backends

### InMemoryBackend (Default)

- **Use case**: Development, testing, or when persistence isn't needed
- **Performance**: Fastest (no I/O)
- **Durability**: None - all data lost on restart
- **Configuration**: Enabled by default, no setup required

```rust
let backend = InMemoryBackend;
backend.init().await?;
```

### SqliteBackend (Optional)

- **Use case**: Production deployments requiring persistence
- **Performance**: Good for < 10k clients
- **Durability**: Full ACID guarantees
- **Configuration**: Requires `--features sqlite`

```rust
let mut backend = SqliteBackend::new("mqtt_broker.db");
backend.init().await?;
```

**SQLite Schema:**
- `sessions` - Client sessions with subscriptions
- `queued_messages` - QoS 1/2 messages for offline clients
- `retained_messages` - Retained messages per topic

## What Gets Persisted

### Sessions
- Client ID
- Persistent flag (clean_session=false)
- List of subscriptions
- Created timestamp

### Queued Messages
- Client ID (foreign key to sessions)
- Topic name
- Payload (binary)
- QoS level (1 or 2 only)
- Retain flag
- Created timestamp

### Retained Messages
- Topic name (primary key)
- Payload (binary)
- QoS level
- Updated timestamp

## Integration with Broker

### SessionManager with Persistence

```rust
use mt_mqtt::persistence::sqlite::SqliteBackend;
use mt_mqtt::session::SessionManager;
use std::sync::Arc;

// Create and initialize backend
let mut backend = SqliteBackend::new("mqtt.db");
backend.init().await?;

// Wrap in Arc for shared ownership
let persistence = Arc::new(backend);

// Create SessionManager with persistence
let session_manager = SessionManager::with_persistence(persistence);
```

### Persistence Operations

**On Client Connect (clean_session=false):**
1. Check in-memory cache for session
2. If not found, query persistence backend
3. Restore session state (subscriptions + queued messages)
4. Populate in-memory cache

**On Client Disconnect (persistent session):**
1. Save session state to persistence
2. Keep session in memory cache
3. Messages queued for offline delivery are persisted

**On Subscribe:**
1. Update in-memory session
2. Persist updated subscription list

**On Message Publish (offline persistent client):**
1. Queue message in memory
2. Persist message to disk
3. Message delivered when client reconnects

**On Message Delivery:**
1. Send queued messages to client
2. Clear persisted messages from database

## Performance Considerations

### Async Operations

All persistence operations are `async` to avoid blocking the broker:

```rust
// Non-blocking write
session_manager.persist_session(&client_id).await?;
```

### Write Strategies

**Immediate Write (Current):**
- Changes written immediately
- Ensures durability
- Slight performance overhead

**Future: Batched Writes**
- Group multiple changes
- Periodic flush (e.g., every 5 seconds)
- Better performance, slight durability trade-off

### Caching Strategy

- **In-memory cache**: Sessions kept in memory for active clients
- **Lazy loading**: Sessions loaded from disk only when needed
- **Write-through**: Changes written to both memory and disk

## Adding New Backends

To add a new persistence backend (e.g., RocksDB, Redis):

1. **Implement the trait:**

```rust
pub struct RocksDbBackend {
    db: rocksdb::DB,
}

#[async_trait]
impl PersistenceBackend for RocksDbBackend {
    async fn save_session(&self, session: &PersistedSession) -> Result<(), PersistenceError> {
        // Serialize session
        let key = format!("session:{}", session.client_id);
        let value = serde_json::to_vec(session)?;
        
        // Write to RocksDB
        self.db.put(key, value)?;
        Ok(())
    }
    // ... implement other methods
}
```

2. **Add as optional dependency:**

```toml
[dependencies]
rocksdb = { version = "0.21", optional = true }

[features]
rocksdb = ["dep:rocksdb"]
```

3. **Use in broker:**

```rust
#[cfg(feature = "rocksdb")]
let backend = RocksDbBackend::new("./data")?;
```

## Migration and Schema Versioning

### Current Approach

SQLite schema is created automatically on first run:
- Tables created if they don't exist
- Safe for new installations

### Future: Migrations

For schema changes:
1. Version table: `CREATE TABLE schema_version (version INT)`
2. Migration scripts: SQL files per version
3. Auto-migrate on startup

```sql
-- Migration: v1 to v2
ALTER TABLE sessions ADD COLUMN last_activity INTEGER;
UPDATE schema_version SET version = 2;
```

## Configuration

### Environment Variables (Future)

```bash
# Persistence type
MQTT_PERSISTENCE=sqlite  # or 'memory', 'rocksdb'

# SQLite-specific
MQTT_SQLITE_PATH=./data/mqtt.db
MQTT_SQLITE_POOL_SIZE=5

# General
MQTT_PERSISTENCE_SYNC_INTERVAL=5  # seconds
```

### Config File (Future)

```toml
[persistence]
backend = "sqlite"
path = "./data/mqtt.db"
pool_size = 5
sync_interval = 5
```

## Monitoring and Observability

### Metrics (Planned)

- `persistence_save_duration_seconds` - Write latency
- `persistence_load_duration_seconds` - Read latency
- `persistence_queue_size` - Pending writes (if batched)
- `persistence_errors_total` - Failed operations

### Logging

```rust
tracing::info!("SQLite persistence initialized at {}", db_path);
tracing::debug!("Persisted session for client {}", client_id);
tracing::warn!("Persistence write failed: {}", error);
```

## Testing

### Unit Tests

Each backend should have tests for all operations:

```rust
#[tokio::test]
async fn test_sqlite_save_and_load_session() {
    let mut backend = SqliteBackend::new(":memory:");
    backend.init().await.unwrap();
    
    let session = PersistedSession {
        client_id: "test".to_string(),
        persistent: true,
        subscriptions: HashSet::from(["topic/1".to_string()]),
    };
    
    backend.save_session(&session).await.unwrap();
    let loaded = backend.load_session("test").await.unwrap();
    assert_eq!(loaded.unwrap().client_id, "test");
}
```

### Integration Tests

Test end-to-end flows:
1. Save session → restart broker → restore session
2. Queue messages → restart → deliver messages
3. Retained messages survive restart

## Troubleshooting

### Common Issues

**Database locked (SQLite):**
- Increase pool size
- Check for long-running transactions
- Consider connection timeout

**Slow persistence operations:**
- Check disk I/O
- Consider batched writes
- Profile with `tracing`

**Data not persisting:**
- Verify feature flag: `--features sqlite`
- Check file permissions
- Review logs for errors

## Future Enhancements

1. **Compression**: Compress payloads before storing
2. **Encryption**: Encrypt sensitive data at rest
3. **Replication**: Multi-node persistence
4. **TTL**: Automatic expiry of old sessions
5. **Vacuum**: Periodic database optimization
6. **Snapshots**: Point-in-time backups

## References

- [SQLx Documentation](https://docs.rs/sqlx/)
- [MQTT 3.1.1 Specification](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html)
- [Rust async-trait](https://docs.rs/async-trait/)
