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
