use tokio::sync::RwLock;
use std::{collections::HashMap, sync::Arc};

#[cfg(test)]
mod test;

pub type DelegateName = &'static str;
pub type Data = String;
pub type Reply = String;

pub struct Delegate<E> {
    listeners: Arc<RwLock<HashMap<DelegateName, Box<dyn Fn(Data) -> Result<Reply, E> + Send + Sync>>>>
}

impl<E> Delegate<E> {
    pub fn new() -> Self {
        Self {
            listeners: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn broadcast(&self, channel: DelegateName, data: Data) -> Result<Reply, E> {
        // Get all listeners
        let listener = self.listeners.read().await;
        let listener_to_channel = listener.get(channel).unwrap();
        
        listener_to_channel(data)
    }

    pub async fn listens<F>(&self, channel: DelegateName, handler: F) 
    where 
        F: Fn(Data) -> Result<Reply, E> + Send + Sync + 'static
    {
        let mut listeners = self.listeners.write().await;
        listeners.insert(channel, Box::new(handler));
    }
}

#[macro_export]
macro_rules! listens {
    ($broker:expr, $channel:expr, $consumer:expr, $method:ident) => {{
        let consumer_clone = $consumer.clone();
        $broker.listens($channel, move |data| consumer_clone.$method(data)).await;
    }};
}
