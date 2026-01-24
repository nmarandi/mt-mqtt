pub mod broker;
mod client;
mod protocol;
mod server;
pub mod topic;

extern crate strum;
extern crate strum_macros;

pub async fn start_broker() -> Result<(), Box<dyn std::error::Error>> {
    server::MqttServer::start().await
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // This test starts an infinite server loop - disabled for unit testing
    #[tokio::test]
    #[ignore]
    async fn simple_mqtt_server_test() {
        start_broker().await.expect("my function");
    }
}
