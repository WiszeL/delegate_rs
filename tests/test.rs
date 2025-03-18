use std::{fmt::Error, sync::Arc};

use delegate::{async_broadcast, broadcast, DelegateManager};
use delegate_rs::{async_listens, listens, delegate};

#[delegate]
struct ConsumerTest {}

impl ConsumerTest {
    pub fn new(delegate_manager: Arc<DelegateManager>) -> Arc<Self> {
        let consumer = Arc::new(Self { delegate_manager });

        listens!(consumer, test_channel);
        listens!(consumer, void_test_channel);
        async_listens!(consumer, async_test_channel);

        consumer
    }

    fn test_channel(&self, data: String) -> Result<String, Error> {
        Ok(format!("Test Channel: {data}."))
    }

    fn void_test_channel(&self, data: &str) -> Result<(), Error> {
        println!("Void test Channel! data: {data}");

        Ok(())
    }

    async fn async_test_channel(&self, data: String) -> Result<String, Error> {
        Ok(format!("Async Test Channel: {data}."))
    }
}

#[delegate]
struct ProducerTest {}

impl ProducerTest {
    pub fn new(delegate_manager: Arc<DelegateManager>) -> Arc<Self> {
        Arc::new(Self { delegate_manager })
    }

    pub async fn produce_test_channel(&self) {
        let hello: String = broadcast!(self, "test_channel", "Hello".to_string()).unwrap();
        let world: String = broadcast!(self, "test_channel", "World".to_string()).unwrap();
        let _: () = broadcast!(self, "void_test_channel", "Void Data").unwrap();

        let hello_world: String = async_broadcast!(self, "async_test_channel", "async hello world!".to_string()).unwrap();

        println!("RESULT: {hello} {world} {hello_world}")
    }
}

#[tokio::test]
async fn broke_test() {
    let delegate_manager = Arc::new(DelegateManager::new());
    let _ = ConsumerTest::new(delegate_manager.clone());
    let producer = ProducerTest::new(delegate_manager.clone());

    producer.produce_test_channel().await;
}
