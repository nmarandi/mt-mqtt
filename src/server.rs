use crate::client::*;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;

const SECURE_TCP_PORT: u32 = 8883;
const UNSECURE_TCP_PORT: u32 = 1883;
const NUM_THREADS: u32 = 4;

pub struct MqttServer {}

impl MqttServer {
    async fn client_spawner(stream: TcpStream) -> Client {
        println!("Spawning a client");
        Client::new(stream)
    }
    pub async fn start() -> Result<(), Box<dyn std::error::Error>> {
        let bind_addr = String::from("0.0.0.0:") + &UNSECURE_TCP_PORT.to_string();
        let mut unsecure_listener =
            TcpListener::bind(bind_addr.clone()).await?;
            println!("Listening on {}", bind_addr);
        loop {
            // Asynchronously wait for an inbound socket.
            let (socket, addr) = unsecure_listener.accept().await?;
            println!("Got a new socket from addr: {:?}", addr);
            // And this is where much of the magic of this server happens. We
            // crucially want all clients to make progress concurrently, rather than
            // blocking one on completion of another. To achieve this we use the
            // `tokio::spawn` function to execute the work in the background.
            //
            // Essentially here we're executing a new task to run concurrently,
            // which will allow all of our clients to be processed concurrently.
            let client = MqttServer::client_spawner(socket).await;
            tokio::spawn(client.run());
        }
    }
}
