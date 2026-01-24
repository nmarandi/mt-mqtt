use crate::{broker::Broker, client::*};
use tokio::net::{TcpListener, TcpStream};

#[allow(dead_code)]
const SECURE_TCP_PORT: u32 = 8883;
const UNSECURE_TCP_PORT: u32 = 1883;

pub struct MqttServer {}

impl MqttServer {
    async fn client_spawner(stream: TcpStream, broker_sender: tokio::sync::mpsc::Sender<crate::broker::BrokerMessage>) -> Client {
        println!("Spawning a client");
        Client::new(stream, broker_sender)
    }

    pub async fn start() -> Result<(), Box<dyn std::error::Error>> {
        // Create and start the broker
        let broker = Broker::new();
        let broker_sender = broker.get_sender();
        
        // Spawn broker task
        tokio::spawn(async move {
            broker.run().await;
        });
        
        let bind_addr = String::from("0.0.0.0:") + &UNSECURE_TCP_PORT.to_string();
        let unsecure_listener = TcpListener::bind(bind_addr.clone()).await?;
        println!("Listening on {}", bind_addr);
        
        loop {
            // Asynchronously wait for an inbound socket.
            let (socket, addr) = unsecure_listener.accept().await?;
            println!("Got a new socket from addr: {:?}", addr);
            
            let broker_sender_clone = broker_sender.clone();
            let client = MqttServer::client_spawner(socket, broker_sender_clone).await;
            tokio::spawn(client.run());
        }
    }
}
