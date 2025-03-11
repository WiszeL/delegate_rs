use std::{fmt::Error, sync::Arc};

use crate::{async_listens, listens, Delegate};

struct ConsumerTest;

impl ConsumerTest {
    pub fn new(delegate: Arc<Delegate<Error>>) -> Arc<Self> {
        let consumer = Arc::new(Self);

        listens!(delegate, consumer, test_channel);
        async_listens!(delegate, consumer, async_test_channel);

        consumer
    }

    fn test_channel(&self, data: String) -> Result<String, Error> {
        let new_str = format!("Do something, answer is: {} modified!", data);
        println!("{}", new_str);

        Ok("".to_string())
    }

    async fn async_test_channel(&self, data: String) -> Result<String, Error> {
        Ok("This is async version!".to_string() + &data)
    }
}

#[tokio::test]
async fn broke_test() {
    let delegate = Arc::new(Delegate::new());
    let _ = ConsumerTest::new(delegate.clone());

    let first: String = delegate.broadcast("test_channel", "first".to_string()).unwrap();
    let second: String = delegate.broadcast("test_channel", "second".to_string()).unwrap();

    let third_async: String = delegate.async_broadcast("async_test_channel", "third".to_string()).await.unwrap();

    println!("Result: {} AND {} ALSO ASYNC {}", first, second, third_async);
}
