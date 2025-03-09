use tokio::sync::RwLock;
use std::{collections::HashMap, sync::Arc};

pub type Channel = &'static str;
pub type Data = String;
pub type Reply = String;

pub struct Delegate {
    listeners: Arc<RwLock<HashMap<Channel, Box<dyn Fn(Data) -> Reply + Send + Sync>>>>
}

impl Delegate {
    pub fn new() -> Self {
        Self {
            listeners: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn broadcast(&self, channel: Channel, data: Data) -> Reply {
        // Get all listeners
        let listener = self.listeners.read().await;
        let listener_to_channel = listener.get(channel).unwrap();
        
        listener_to_channel(data)
    }

    pub async fn listens<F>(&self, channel: Channel, handler: F) 
    where 
        F: Fn(Data) -> Reply + Send + Sync + 'static
    {
        let mut listeners = self.listeners.write().await;
        listeners.insert(channel, Box::new(handler));
    }
}

#[macro_export]
macro_rules! subscribe {
    ($broker:expr, $channel:expr, $consumer:expr, $method:ident) => {{
        let consumer_clone = $consumer.clone();
        $broker.listens($channel, move |data| consumer_clone.$method(data)).await;
    }};
}