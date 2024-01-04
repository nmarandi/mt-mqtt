use crate::client::Client;
use std::collections::HashMap;
use tokio::sync::mpsc::{self, Receiver, Sender};

#[derive(Debug)]
pub enum BrokerMessage {
    AddPublisher(Client),
    AddSubscriber(Client),
    RemovePublisher(Client),
    RemoveSubscriber(Client),
}


pub struct Broker {
    publisher: Vec<Client>,
    subscriber: HashMap<Vec<Client>>,
    sender: Sender<BrokerMessage>,
    receiver: Receiver<BrokerMessage>
}

impl Broker {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(100);

        self { publisher: Vec::new(), subscriber: HashMap::new(), sender, receiver}
    }
}