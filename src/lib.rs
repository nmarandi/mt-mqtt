pub mod broker;
mod client;
mod message_state;
mod packet_id;
pub mod protocol;
mod server;
pub mod topic;

extern crate strum;
extern crate strum_macros;

pub async fn start_broker(bind_addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    server::MqttServer::start(bind_addr).await
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // This test starts an infinite server loop - disabled for unit testing
    #[tokio::test]
    #[ignore]
    async fn simple_mqtt_server_test() {
        start_broker("localhost:1883").await.expect("my function");
    }
}
