use std::sync::Arc;

use delegate_rs::{
    Error, async_bind_delegate, async_broadcast_delegate, bind_delegate, broadcast_delegate,
};

pub type AppResult<T> = Result<T, Error>;

struct ConsumerTest;

impl ConsumerTest {
    pub fn new() -> Arc<Self> {
        let consumer = Arc::new(Self);

        bind_delegate!(consumer, test_channel);
        async_bind_delegate!(consumer, async_test_channel);

        consumer
    }

    fn test_channel(&self, data: &str) -> Result<String, Error> {
        Ok(format!("Test Channel: {data}."))
    }

    async fn async_test_channel(&self, data: String) -> Result<(), Error> {
        // Simulate async work with 3 second delay
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        println!("Async Test Channel: {data}.");

        Ok(())
    }
}

struct ProducerTest;

impl ProducerTest {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }

    pub async fn produce_test_channel(&self) -> AppResult<()> {
        let hello = broadcast_delegate!("test_channel", "hello", String)?;
        println!("Result Hello: {hello}");

        async_broadcast_delegate!("async_test_channel", "async hello world!".to_string())?;

        let world = broadcast_delegate!("test_channel", "world", String)?;
        println!("Result World: {world}");

        Ok(())
    }
}

#[tokio::test]
async fn broke_test() {
    delegate_rs::init();

    let _ = ConsumerTest::new();
    let producer = ProducerTest::new();

    producer.produce_test_channel().await.unwrap();
}
