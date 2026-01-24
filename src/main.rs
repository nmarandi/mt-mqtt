use mt_mqtt::start_broker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting MT-MQTT Broker...");
    start_broker().await
}
