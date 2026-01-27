/// SQLite persistence backend implementation

use super::{PersistenceBackend, PersistenceError, PersistedMessage, PersistedSession};
use crate::broker::PublishMessage;
use async_trait::async_trait;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::collections::HashSet;

pub struct SqliteBackend {
    pool: Option<SqlitePool>,
    db_path: String,
}

impl SqliteBackend {
    pub fn new(db_path: &str) -> Self {
        Self {
            pool: None,
            db_path: db_path.to_string(),
        }
    }

    async fn create_tables(&self) -> Result<(), PersistenceError> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            PersistenceError::Database("Pool not initialized".to_string())
        })?;

        // Create sessions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
                client_id TEXT PRIMARY KEY,
                persistent INTEGER NOT NULL,
                subscriptions TEXT NOT NULL,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            )
            "#,
        )
        .execute(pool)
        .await
        .map_err(|e| PersistenceError::Database(e.to_string()))?;

        // Create queued_messages table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS queued_messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                client_id TEXT NOT NULL,
                topic TEXT NOT NULL,
                payload BLOB NOT NULL,
                qos INTEGER NOT NULL,
                retain INTEGER NOT NULL,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                FOREIGN KEY (client_id) REFERENCES sessions(client_id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(pool)
        .await
        .map_err(|e| PersistenceError::Database(e.to_string()))?;

        // Create index on client_id for faster queries
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_queued_messages_client_id 
            ON queued_messages(client_id)
            "#,
        )
        .execute(pool)
        .await
        .map_err(|e| PersistenceError::Database(e.to_string()))?;

        // Create retained_messages table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS retained_messages (
                topic TEXT PRIMARY KEY,
                payload BLOB NOT NULL,
                qos INTEGER NOT NULL,
                updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            )
            "#,
        )
        .execute(pool)
        .await
        .map_err(|e| PersistenceError::Database(e.to_string()))?;

        Ok(())
    }
}

#[async_trait]
impl PersistenceBackend for SqliteBackend {
    async fn init(&mut self) -> Result<(), PersistenceError> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&format!("sqlite:{}", self.db_path))
            .await
            .map_err(|e| PersistenceError::Database(e.to_string()))?;

        self.pool = Some(pool);
        self.create_tables().await?;

        tracing::info!("SQLite persistence initialized at {}", self.db_path);
        Ok(())
    }

    async fn save_session(&self, session: &PersistedSession) -> Result<(), PersistenceError> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            PersistenceError::Database("Pool not initialized".to_string())
        })?;

        let subscriptions_json = serde_json::to_string(&session.subscriptions)
            .map_err(|e| PersistenceError::Serialization(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO sessions (client_id, persistent, subscriptions)
            VALUES (?, ?, ?)
            ON CONFLICT(client_id) DO UPDATE SET
                persistent = excluded.persistent,
                subscriptions = excluded.subscriptions
            "#,
        )
        .bind(&session.client_id)
        .bind(session.persistent as i32)
        .bind(subscriptions_json)
        .execute(pool)
        .await
        .map_err(|e| PersistenceError::Database(e.to_string()))?;

        Ok(())
    }

    async fn load_session(&self, client_id: &str) -> Result<Option<PersistedSession>, PersistenceError> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            PersistenceError::Database("Pool not initialized".to_string())
        })?;

        let row: Option<(String, i32, String)> = sqlx::query_as(
            "SELECT client_id, persistent, subscriptions FROM sessions WHERE client_id = ?"
        )
        .bind(client_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| PersistenceError::Database(e.to_string()))?;

        match row {
            Some((client_id, persistent, subscriptions_json)) => {
                let subscriptions: HashSet<String> = serde_json::from_str(&subscriptions_json)
                    .map_err(|e| PersistenceError::Serialization(e.to_string()))?;

                Ok(Some(PersistedSession {
                    client_id,
                    persistent: persistent != 0,
                    subscriptions,
                }))
            }
            None => Ok(None),
        }
    }

    async fn delete_session(&self, client_id: &str) -> Result<(), PersistenceError> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            PersistenceError::Database("Pool not initialized".to_string())
        })?;

        sqlx::query("DELETE FROM sessions WHERE client_id = ?")
            .bind(client_id)
            .execute(pool)
            .await
            .map_err(|e| PersistenceError::Database(e.to_string()))?;

        Ok(())
    }

    async fn queue_message(&self, message: &PersistedMessage) -> Result<(), PersistenceError> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            PersistenceError::Database("Pool not initialized".to_string())
        })?;

        sqlx::query(
            r#"
            INSERT INTO queued_messages (client_id, topic, payload, qos, retain)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&message.client_id)
        .bind(&message.topic)
        .bind(&message.payload)
        .bind(message.qos as i32)
        .bind(message.retain as i32)
        .execute(pool)
        .await
        .map_err(|e| PersistenceError::Database(e.to_string()))?;

        Ok(())
    }

    async fn get_queued_messages(&self, client_id: &str) -> Result<Vec<PersistedMessage>, PersistenceError> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            PersistenceError::Database("Pool not initialized".to_string())
        })?;

        let rows: Vec<(String, String, Vec<u8>, i32, i32)> = sqlx::query_as(
            "SELECT client_id, topic, payload, qos, retain FROM queued_messages WHERE client_id = ? ORDER BY id"
        )
        .bind(client_id)
        .fetch_all(pool)
        .await
        .map_err(|e| PersistenceError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|(client_id, topic, payload, qos, retain)| PersistedMessage {
                client_id,
                topic,
                payload,
                qos: qos as u8,
                retain: retain != 0,
            })
            .collect())
    }

    async fn delete_queued_messages(&self, client_id: &str) -> Result<(), PersistenceError> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            PersistenceError::Database("Pool not initialized".to_string())
        })?;

        sqlx::query("DELETE FROM queued_messages WHERE client_id = ?")
            .bind(client_id)
            .execute(pool)
            .await
            .map_err(|e| PersistenceError::Database(e.to_string()))?;

        Ok(())
    }

    async fn save_retained_message(&self, topic: &str, message: &PublishMessage) -> Result<(), PersistenceError> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            PersistenceError::Database("Pool not initialized".to_string())
        })?;

        sqlx::query(
            r#"
            INSERT INTO retained_messages (topic, payload, qos)
            VALUES (?, ?, ?)
            ON CONFLICT(topic) DO UPDATE SET
                payload = excluded.payload,
                qos = excluded.qos,
                updated_at = strftime('%s', 'now')
            "#,
        )
        .bind(topic)
        .bind(&message.payload)
        .bind(message.qos as i32)
        .execute(pool)
        .await
        .map_err(|e| PersistenceError::Database(e.to_string()))?;

        Ok(())
    }

    async fn load_retained_message(&self, topic: &str) -> Result<Option<PublishMessage>, PersistenceError> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            PersistenceError::Database("Pool not initialized".to_string())
        })?;

        let row: Option<(String, Vec<u8>, i32)> = sqlx::query_as(
            "SELECT topic, payload, qos FROM retained_messages WHERE topic = ?"
        )
        .bind(topic)
        .fetch_optional(pool)
        .await
        .map_err(|e| PersistenceError::Database(e.to_string()))?;

        match row {
            Some((topic, payload, qos)) => Ok(Some(PublishMessage {
                topic,
                payload,
                qos: qos as u8,
                retain: true,
            })),
            None => Ok(None),
        }
    }

    async fn delete_retained_message(&self, topic: &str) -> Result<(), PersistenceError> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            PersistenceError::Database("Pool not initialized".to_string())
        })?;

        sqlx::query("DELETE FROM retained_messages WHERE topic = ?")
            .bind(topic)
            .execute(pool)
            .await
            .map_err(|e| PersistenceError::Database(e.to_string()))?;

        Ok(())
    }

    async fn get_all_retained_messages(&self) -> Result<Vec<(String, PublishMessage)>, PersistenceError> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            PersistenceError::Database("Pool not initialized".to_string())
        })?;

        let rows: Vec<(String, Vec<u8>, i32)> = sqlx::query_as(
            "SELECT topic, payload, qos FROM retained_messages"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| PersistenceError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|(topic, payload, qos)| {
                let message = PublishMessage {
                    topic: topic.clone(),
                    payload,
                    qos: qos as u8,
                    retain: true,
                };
                (topic, message)
            })
            .collect())
    }
}
