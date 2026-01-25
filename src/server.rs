use crate::{broker::Broker, client::*};
use tokio::net::{TcpListener, TcpStream};
use std::time::Instant;

pub struct MqttServer {}

impl MqttServer {
    fn client_spawner(stream: TcpStream, broker_sender: tokio::sync::mpsc::Sender<crate::broker::BrokerMessage>) -> Client {
        Client::new(stream, broker_sender)
    }

    pub async fn start(bind_addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Create and start the broker
        let broker = Broker::new();
        let broker_sender = broker.get_sender();
        
        // Spawn broker task
        tokio::spawn(async move {
            broker.run().await;
        });
        
        let unsecure_listener = TcpListener::bind(bind_addr).await?;
        tracing::info!("Listening on {}", bind_addr);
        
        loop {
            let accept_start = Instant::now();
            // Asynchronously wait for an inbound socket.
            let (socket, addr) = unsecure_listener.accept().await?;
            let accept_time = accept_start.elapsed();
            tracing::warn!("TIMING: accept took {:?} for {:?}", accept_time, addr);
            
            let setup_start = Instant::now();
            
            let broker_sender_clone = broker_sender.clone();
            let client = MqttServer::client_spawner(socket, broker_sender_clone);
            tokio::spawn(client.run());
            let setup_time = setup_start.elapsed();
            tracing::warn!("TIMING: client setup+spawn took {:?}", setup_time);
        }
    }
}
