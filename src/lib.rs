#![feature(arbitrary_enum_discriminant)]
mod client;
mod definitions;
mod frame;
mod packet;
mod server;

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
