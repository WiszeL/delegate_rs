use dashmap::DashMap;
use derive_more::Display;
use futures::future::BoxFuture;
use thiserror::Error;
use std::{any::Any, future::Future, sync::Arc};

type Data = Box<dyn Any + Send + Sync>;
type Reply = Box<dyn Any + Send + Sync>;

#[derive(Debug, Display, Error)]
pub enum Error {
    Mismatched(String),
    NoListener(String),
    ExternalError(Box<dyn std::error::Error + Send + Sync>),
}

pub enum Listener {
    Sync(Box<dyn Fn(Data) -> Result<Reply, Error> + Send + Sync>),
    Async(Box<dyn Fn(Data) -> BoxFuture<'static, Result<Reply, Error>> + Send + Sync>),
}

pub struct DelegateManager {
    delegates: Arc<DashMap<&'static str, Listener>>,
}

impl DelegateManager {
    pub fn new() -> Self {
        Self {
            delegates: Arc::new(DashMap::new()),
        }
    }

    /// Broadcast to a listener that is expected to be synchronous.
    pub fn broadcast<'a, R, D>(&self, name: &'a str, data: D) -> Result<R, Error>
    where
        R: Any + Send + Sync,
        D: Any + Send + Sync,
    {
        let listener = self
            .delegates
            .get(name)
            .ok_or(Error::NoListener(format!("No listener registered for {name}")))?;

        let reply_box = match &*listener {
            Listener::Sync(sync_fn) => (sync_fn)(Box::new(data) as Data),
            Listener::Async(_) => Err(Error::Mismatched(format!("Called broadcast_async on sync listener {name}"))),
        }?;

        match reply_box.downcast::<R>() {
            Ok(boxed) => Ok(*boxed),
            Err(_) => Err(Error::Mismatched(format!("Reply type mismatch on {name}"))),
        }
    }

    /// Broadcast to a listener, allowing async listeners.
    /// If the listener is synchronous, its result is returned immediately BUT NOT RECOMMENDED TO USE THIS
    pub async fn async_broadcast<'a, R, D>(&self, name: &'a str, data: D) -> Result<R, Error>
    where
        R: Any + Send + Sync,
        D: Any + Send + Sync,
    {
        let listener = self
            .delegates
            .get(name)
            .ok_or(Error::NoListener(format!("No listener registered for {name}")))?;

        let reply_box = match &*listener {
            Listener::Async(async_fn) => async_fn(Box::new(data) as Data).await,
            Listener::Sync(_) => Err(Error::Mismatched(format!("Called broadcast_sync on async listener {name}"))),
        }?;

        match reply_box.downcast::<R>() {
            Ok(boxed) => Ok(*boxed),
            Err(_) => Err(Error::Mismatched(format!("Reply type mismatch on {name}."))),
        }
    }

    // Register a synchronous listener.
    pub fn listens<D, R, E, F>(&self, name: &'static str, handler: F)
    where
        D: Any + Send + Sync,
        R: Any + Send + Sync,
        E: std::error::Error + Send + Sync + 'static,
        F: Fn(D) -> Result<R, E> + Send + Sync + 'static,
    {
        self.delegates.insert(
            name,
            Listener::Sync(Box::new(move |data: Data| {
                let boxed_d = data
                    .downcast::<D>()
                    .map_err(|_| Error::Mismatched(format!("Data type mismatch in listens {name}")))?;
                let r = handler(*boxed_d)
                    .map_err(|err| Error::ExternalError(Box::new(err)))?;
                Ok(Box::new(r) as Reply)
            })),
        );
    }

    // Register an async listener.
    pub fn async_listens<D, R, E, F, Fut>(&self, name: &'static str, handler: F)
    where
        D: Any + Send + Sync,
        R: Any + Send + Sync,
        E: std::error::Error + Send + Sync + 'static,
        F: Fn(D) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<R, E>> + Send + 'static,
    {
        // Wrap the handler in an Arc so it can be cloned inside the async closure.
        let handler = Arc::new(handler);
        self.delegates.insert(
            name,
            Listener::Async(Box::new(move |data: Data| {
                let handler = Arc::clone(&handler);
                Box::pin(async move {
                    let boxed_d = data
                        .downcast::<D>()
                        .map_err(|_| Error::Mismatched(format!("Data type mismatch in listens {name}")))?;
                    let r = handler(*boxed_d)
                        .await
                        .map_err(|err| Error::ExternalError(Box::new(err)))?;
                    Ok(Box::new(r) as Reply)
                })
            })),
        );
    }
}

#[macro_export]
macro_rules! listens {
    ($consumer:expr, $method:ident) => {{
        let consumer_clone = $consumer.clone();
        $consumer.get_delegate_manager().listens(stringify!($method), move |data| consumer_clone.$method(data));
    }};
}

#[macro_export]
macro_rules! async_listens {
    ($consumer:expr, $method:ident) => {{
        let consumer_clone = $consumer.clone();
        $consumer.get_delegate_manager().async_listens(stringify!($method), move |data| {
            // Clone again inside the closure so the async block doesn't borrow consumer_clone.
            let consumer_inner = consumer_clone.clone();
            async move { consumer_inner.$method(data).await }
        });
    }};
}

#[macro_export]
macro_rules! broadcast {
    ($instance:expr, $delegate_name:expr, $data:expr) => {
        $instance.get_delegate_manager().broadcast($delegate_name, $data)
    };
}

#[macro_export]
macro_rules! async_broadcast {
    ($instance:expr, $delegate_name:expr, $data:expr) => {
        $instance.get_delegate_manager().async_broadcast($delegate_name, $data).await
    };
}
