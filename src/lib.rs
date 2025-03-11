use dashmap::DashMap;
use std::{any::Any, sync::Arc};

#[cfg(test)]
mod test;

type Data = Box<dyn Any + Send + Sync>;
type Reply = Box<dyn Any + Send + Sync>;

pub struct Delegate<E> {
    listeners: Arc<DashMap<&'static str, Box<dyn Fn(Data) -> Result<Reply, E> + Send + Sync>>>
}

impl<E> Delegate<E> {
    pub fn new() -> Self {
        Self {
            listeners: Arc::new(DashMap::new()),
        }
    }

    pub fn broadcast<D, R>(&self, name: &'static str, data: D) -> Result<R, E>
    where
        D: Any + Send + Sync,
        R: Any + Send + Sync,
    {
        let listener = self.listeners.get(name).expect("No listener registered for this name");
        
        // Box the data to match the expected Data type
        let reply_box = listener(Box::new(data) as Data)?;
        
        // Downcast the reply to the expected type R
        match reply_box.downcast::<R>() {
            Ok(boxed) => Ok(*boxed),
            Err(_) => panic!("Reply type mismatch"),
        }
    }

    pub fn listens<D, R, F>(&self, name: &'static str, handler: F)
    where
        D: Any + Send + Sync,
        R: Any + Send + Sync,
        F: Fn(D) -> Result<R, E> + Send + Sync + 'static,
    {
        self.listeners.insert(name, Box::new(move |data: Data| {
            // Downcast the boxed data to the expected type.
            let boxed_d = data.downcast::<D>().expect("Data type mismatch in listens");
            let r = handler(*boxed_d)?;

            // Box the result as the expected reply type.
            Ok(Box::new(r) as Reply)
        }));
    }
}

#[macro_export]
macro_rules! listens {
    ($delegate:expr, $consumer:expr, $method:ident) => {{
        let consumer_clone = $consumer.clone();
        $delegate.listens(stringify!($method), move |data| consumer_clone.$method(data));
    }};
}
