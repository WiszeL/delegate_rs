use tokio::sync::RwLock;
use std::{any::Any, collections::HashMap, sync::Arc};

#[cfg(test)]
mod test;

pub type DelegateName = &'static str;
pub type Data = Box<dyn Any + Send + Sync>;
pub type Reply = Box<dyn Any + Send + Sync>;

pub struct Delegate<E> {
    listeners: Arc<RwLock<HashMap<DelegateName, Box<dyn Fn(Data) -> Result<Reply, E> + Send + Sync>>>>
}

impl<E> Delegate<E> {
    pub fn new() -> Self {
        Self {
            listeners: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn broadcast(&self, name: DelegateName, data: Data) -> Result<Reply, E> {
        // Get all listeners
        let listener = self.listeners.read().await;
        let listener_to_name = listener.get(name).unwrap();
        
        listener_to_name(data)
    }

    pub async fn listens<F>(&self, name: DelegateName, handler: F) 
    where 
        F: Fn(Data) -> Result<Reply, E> + Send + Sync + 'static
    {
        let mut listeners = self.listeners.write().await;
        listeners.insert(name, Box::new(handler));
    }
}

#[macro_export]
macro_rules! listens {
    ($consumer:expr, $method:ident) => {{
        let consumer_clone = $consumer.clone();
        $consumer.delegate.listens(stringify!($method), move |data| consumer_clone.$method(data)).await;
    }};
}

#[macro_export]
macro_rules! make_data {
    ($data:expr) => {
        Box::new($data) as Data
    };
}

#[macro_export]
macro_rules! make_reply {
    ($data:expr) => {
        Ok(Box::new($data) as Reply)
    };
}
