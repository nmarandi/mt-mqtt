use mt_mqtt::start_broker;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber with env filter
    // Set RUST_LOG=debug for verbose output, RUST_LOG=info for normal, RUST_LOG=warn for quiet
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mt_mqtt=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let bind_addr = if args.len() > 1 {
        args[1].clone()
    } else {
        "[::]:1883".to_string()  // IPv6 dual-stack accepts both IPv4 and IPv6
    };

    tracing::info!("Starting MT-MQTT Broker on {}...", bind_addr);
    start_broker(&bind_addr).await
}
