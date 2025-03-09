use std::{fmt::Error, sync::Arc};

use crate::{listens, Data, Delegate, Reply};

struct ConsumerTest;

impl ConsumerTest {
    pub async fn new(broker: Arc<Delegate<Error>>) -> Arc<Self> {
        let consumer = Arc::new(Self);

        listens!(broker, "test_channel", consumer, do_something);

        consumer
    }

    fn do_something(&self, data: Data) -> Result<Reply, Error> {
        let new_str = format!("Do something, answer is: {} modified!", data);
        println!("{}", new_str);

        Ok(new_str + " reply!")
    }
}

#[tokio::test]
async fn broke_test() {
    let broker = Arc::new(Delegate::new());
    let _ = ConsumerTest::new(broker.clone()).await;

    let first = broker.broadcast("test_channel", "first".to_string()).await.unwrap();
    let second = broker.broadcast("test_channel", "second".to_string()).await.unwrap();

    println!("Result: {} AND {}", first, second);
}
