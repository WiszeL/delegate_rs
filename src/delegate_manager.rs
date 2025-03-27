use dashmap::DashMap;
use futures::future::BoxFuture;
use std::{
    any::{Any, TypeId},
    future::Future,
    sync::Arc,
};
use thiserror::Error;

type BindArg = Box<dyn Any + Send + Sync>;
type BindReturn = Box<dyn Any + Send + Sync>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Mismatched(String),

    #[error("No listener for delegate name {0}")]
    NoListener(String),

    #[error(transparent)]
    External(Box<dyn std::error::Error + Send + Sync>)
}

pub enum Listener {
    Sync {
        type_id_arg: TypeId,
        type_id_return: TypeId,
        handler: Box<dyn Fn(BindArg) -> Result<BindReturn, Error> + Send + Sync>,
    },
    Async {
        type_id_arg: TypeId,
        type_id_return: TypeId,
        handler: Box<dyn Fn(BindArg) -> BoxFuture<'static, Result<BindReturn, Error>> + Send + Sync>,
    },
}

pub struct DelegateManager {
    single: Arc<DashMap<&'static str, Listener>>,

    pub type_registry: Arc<DashMap<&'static str, (TypeId, TypeId)>>,
}

impl Default for DelegateManager {
    fn default() -> Self {
        Self {
            single: Arc::new(DashMap::new()),
            type_registry: Arc::new(DashMap::new()),
        }
    }
}

impl DelegateManager {
    /// Broadcast to a listener that is expected to be synchronous.
    pub fn broadcast<A, R>(&self, name: &str, arg: A) -> Result<R, Error>
    where
        A: Any + Send + Sync,
        R: Any + Send + Sync,
    {
        let listener = self
            .single
            .get(name)
            .ok_or(Error::NoListener(name.to_string()))?;

        let (type_id_arg, type_id_return) = match &*listener {
            Listener::Sync {
                type_id_arg,
                type_id_return,
                ..
            } => (type_id_arg, type_id_return),
            Listener::Async { .. } => {
                return Err(Error::Mismatched(format!(
                    "Called sync broadcast on async listener {name}"
                )));
            }
        };

        Self::check_type::<A>(name, type_id_arg, "arg")?;
        Self::check_type::<R>(name, type_id_return, "return")?;

        let reply_box = match &*listener {
            Listener::Sync { handler, .. } => handler(Box::new(arg) as BindArg),
            _ => unreachable!(),
        }?;

        match reply_box.downcast::<R>() {
            Ok(boxed) => Ok(*boxed),
            Err(_) => Err(Error::Mismatched(format!("Reply type mismatch on {name}"))),
        }
    }

    /// Broadcast to a listener, allowing async listeners.
    pub async fn async_broadcast<A, R>(&self, name: &str, data: A) -> Result<R, Error>
    where
        A: Any + Send + Sync,
        R: Any + Send + Sync,
    {
        let listener = self
            .single
            .get(name)
            .ok_or(Error::NoListener(name.to_string()))?;

        let (type_id_arg, type_id_return) = match &*listener {
            Listener::Async {
                type_id_arg,
                type_id_return,
                ..
            } => (type_id_arg, type_id_return),
            Listener::Sync { .. } => {
                return Err(Error::Mismatched(format!(
                    "Called async broadcast on sync listener {name}"
                )));
            }
        };

        Self::check_type::<A>(name, type_id_arg, "arg")?;
        Self::check_type::<R>(name, type_id_return, "return")?;

        let reply_box = match &*listener {
            Listener::Async { handler, .. } => handler(Box::new(data) as BindArg).await,
            _ => unreachable!(),
        }?;

        match reply_box.downcast::<R>() {
            Ok(boxed) => Ok(*boxed),
            Err(_) => Err(Error::Mismatched(format!("Reply type mismatch on {name}."))),
        }
    }

    // Register a synchronous listener.
    pub fn bind<D, R, E, F>(&self, name: &'static str, handler: F)
    where
        D: Any + Send + Sync,
        R: Any + Send + Sync,
        E: std::error::Error + Send + Sync + 'static,
        F: Fn(D) -> Result<R, E> + Send + Sync + 'static,
    {
        let type_id_arg = TypeId::of::<D>();
        let type_id_return = TypeId::of::<R>();

        self.type_registry.insert(name, (TypeId::of::<D>(), TypeId::of::<R>()));
        self.single.insert(
            name,
            Listener::Sync {
                type_id_arg,
                type_id_return,
                handler: Box::new(move |data: BindArg| {
                    let boxed_d = data.downcast::<D>().unwrap();
                    let r = handler(*boxed_d)
                        .map_err(|err| Error::External(Box::new(err)))?;

                    Ok(Box::new(r) as BindReturn)
                }),
            },
        );
    }

    // Register an async listener.
    pub fn async_bind<D, R, E, F, Fut>(&self, name: &'static str, handler: F)
    where
        D: Any + Send + Sync,
        R: Any + Send + Sync,
        E: std::error::Error + Send + Sync + 'static,
        F: Fn(D) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<R, E>> + Send + 'static,
    {
        let type_id_d = TypeId::of::<D>();
        let type_id_r = TypeId::of::<R>();

        let handler = Arc::new(handler);

        self.type_registry.insert(name, (TypeId::of::<D>(), TypeId::of::<R>()));
        self.single.insert(
            name,
            Listener::Async {
                type_id_arg: type_id_d,
                type_id_return: type_id_r,
                handler: Box::new(move |data: BindArg| {
                    let handler = Arc::clone(&handler);

                    Box::pin(async move {
                        let boxed_d = data.downcast::<D>().unwrap();
                        let r = handler(*boxed_d)
                            .await
                            .map_err(|err| Error::External(Box::new(err)))?;

                        Ok(Box::new(r) as BindReturn)
                    })
                }),
            },
        );
    }

    fn check_type<T>(delegate_name: &str, type_id: &TypeId, on: &'static str) -> Result<(), Error>
    where
        T: 'static,
    {
        if TypeId::of::<T>() != *type_id {
            Err(Error::Mismatched(format!(
                "Data type mismatch for {delegate_name} on {on}"
            )))
        } else {
            Ok(())
        }
    }

    pub fn get_types(&self, name: &str) -> Option<(TypeId, TypeId)> {
        self.single.get(name).map(|entry| match entry.value() {
            Listener::Sync { type_id_arg, type_id_return, .. } => (*type_id_arg, *type_id_return),
            Listener::Async { type_id_arg, type_id_return, .. } => (*type_id_arg, *type_id_return),
        })
    }
}
