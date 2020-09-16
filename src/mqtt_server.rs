use tokio::net::TcpListener;
use tokio::prelude::*;

const secure_tcp_port: u32 = 8883;
const unsecure_tcp_port: u32 = 1883; 

pub struct MqttServer {

}

impl mqttServer {
    async fn start() -> Result<(), Box<dyn std::error::Error>> {
        let mut unsecure_listener = TcpListener::bind("0.0.0.0:" + unsecure_tcp_port.to_string()).await?;
    }
}