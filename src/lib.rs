use tokio::sync::RwLock;
use std::{any::Any, collections::HashMap, sync::Arc};

#[cfg(test)]
mod test;

mod macros;

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
