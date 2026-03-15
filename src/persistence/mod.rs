/// Persistence layer abstraction for session and message storage
///
/// This trait allows swapping between different storage backends (SQLite, RocksDB, etc.)
/// without changing the broker code.
use crate::broker::PublishMessage;
use async_trait::async_trait;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct PersistedSession {
    pub client_id: String,
    pub persistent: bool,
    pub subscriptions: HashSet<String>,
}

#[derive(Debug, Clone)]
pub struct PersistedMessage {
    pub client_id: String,
    pub topic: String,
    pub payload: Vec<u8>,
    pub qos: u8,
    pub retain: bool,
}

/// Trait for persistence backends
#[async_trait]
pub trait PersistenceBackend: Send + Sync {
    /// Initialize the persistence backend
    async fn init(&mut self) -> Result<(), PersistenceError>;

    /// Save a session
    async fn save_session(&self, session: &PersistedSession) -> Result<(), PersistenceError>;

    /// Load a session by client_id
    async fn load_session(&self, client_id: &str) -> Result<Option<PersistedSession>, PersistenceError>;

    /// Delete a session
    async fn delete_session(&self, client_id: &str) -> Result<(), PersistenceError>;

    /// Queue a message for offline delivery
    async fn queue_message(&self, message: &PersistedMessage) -> Result<(), PersistenceError>;

    /// Get all queued messages for a client
    async fn get_queued_messages(&self, client_id: &str) -> Result<Vec<PersistedMessage>, PersistenceError>;

    /// Delete queued messages for a client
    async fn delete_queued_messages(&self, client_id: &str) -> Result<(), PersistenceError>;

    /// Save a retained message
    async fn save_retained_message(&self, topic: &str, message: &PublishMessage) -> Result<(), PersistenceError>;

    /// Load a retained message
    async fn load_retained_message(&self, topic: &str) -> Result<Option<PublishMessage>, PersistenceError>;

    /// Delete a retained message
    async fn delete_retained_message(&self, topic: &str) -> Result<(), PersistenceError>;

    /// Get all retained messages
    async fn get_all_retained_messages(&self) -> Result<Vec<(String, PublishMessage)>, PersistenceError>;
}

#[derive(Debug, thiserror::Error)]
pub enum PersistenceError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Not found")]
    NotFound,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// In-memory persistence backend (default, no disk persistence)
pub struct InMemoryBackend;

#[async_trait]
impl PersistenceBackend for InMemoryBackend {
    async fn init(&mut self) -> Result<(), PersistenceError> {
        Ok(())
    }

    async fn save_session(&self, _session: &PersistedSession) -> Result<(), PersistenceError> {
        Ok(()) // No-op for in-memory
    }

    async fn load_session(&self, _client_id: &str) -> Result<Option<PersistedSession>, PersistenceError> {
        Ok(None) // Always return None for in-memory
    }

    async fn delete_session(&self, _client_id: &str) -> Result<(), PersistenceError> {
        Ok(())
    }

    async fn queue_message(&self, _message: &PersistedMessage) -> Result<(), PersistenceError> {
        Ok(())
    }

    async fn get_queued_messages(&self, _client_id: &str) -> Result<Vec<PersistedMessage>, PersistenceError> {
        Ok(Vec::new())
    }

    async fn delete_queued_messages(&self, _client_id: &str) -> Result<(), PersistenceError> {
        Ok(())
    }

    async fn save_retained_message(&self, _topic: &str, _message: &PublishMessage) -> Result<(), PersistenceError> {
        Ok(())
    }

    async fn load_retained_message(&self, _topic: &str) -> Result<Option<PublishMessage>, PersistenceError> {
        Ok(None)
    }

    async fn delete_retained_message(&self, _topic: &str) -> Result<(), PersistenceError> {
        Ok(())
    }

    async fn get_all_retained_messages(&self) -> Result<Vec<(String, PublishMessage)>, PersistenceError> {
        Ok(Vec::new())
    }
}

// SQLite implementation (behind feature flag for optional compilation)
#[cfg(feature = "sqlite")]
pub mod sqlite;
