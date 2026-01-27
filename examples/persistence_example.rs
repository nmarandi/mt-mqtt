/// Example of using MT-MQTT with SQLite persistence
/// 
/// This example shows how to configure the broker with SQLite-backed persistence
/// for sessions, queued messages, and retained messages.

#[cfg(feature = "sqlite")]
use mt_mqtt::persistence::sqlite::SqliteBackend;
use mt_mqtt::persistence::{InMemoryBackend, PersistenceBackend};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create persistence backend
    #[cfg(feature = "sqlite")]
    let persistence: Arc<dyn PersistenceBackend> = {
        let mut backend = SqliteBackend::new("mqtt_broker.db");
        backend.init().await?;
        Arc::new(backend)
    };

    #[cfg(not(feature = "sqlite"))]
    let persistence: Arc<dyn PersistenceBackend> = {
        let mut backend = InMemoryBackend;
        backend.init().await?;
        Arc::new(backend)
    };

    println!("MT-MQTT Broker Persistence Example");
    println!("===================================");
    
    #[cfg(feature = "sqlite")]
    println!("✓ Using SQLite persistence at: mqtt_broker.db");
    #[cfg(not(feature = "sqlite"))]
    println!("✗ Using in-memory persistence (sessions will not survive restart)");
    
    println!("\nPersistence backend initialized successfully!");
    println!("\nThis example shows how to create a persistence backend.");
    println!("To integrate with the broker, pass the backend to SessionManager::with_persistence()");
    
    println!("\n# To enable SQLite persistence:");
    println!("  cargo run --example persistence_example --features sqlite");
    
    println!("\n# To use in your broker:");
    println!("  let persistence = Arc::new(SqliteBackend::new(\"mqtt.db\"));");
    println!("  persistence.init().await?;");
    println!("  let session_manager = SessionManager::with_persistence(persistence);");
    
    // Demonstrate basic operations
    println!("\n# Testing persistence operations...");
    
    // Note: Full broker integration would happen here
    // This is just demonstrating the persistence layer setup
    
    Ok(())
}
