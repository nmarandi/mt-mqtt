mod client;
mod definitions;
mod frame;
mod packet;
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
    #[tokio::test]
    async fn simple_mqtt_server_test() {
        start_broker().await.expect("my function");
    }
}
