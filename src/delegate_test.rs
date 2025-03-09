use std::sync::Arc;

use crate::{delegate::{Delegate, Data, Reply}, subscribe};

struct ConsumerTest;

impl ConsumerTest {
    pub async fn new(broker: Arc<Delegate>) -> Arc<Self> {
        let consumer = Arc::new(Self);

        // broker.subcribe("test_channel", move |data| consumer_clone.do_something(data)).await;
        self::subscribe!(broker, "test_channel", consumer, do_something);

        consumer
    }

    fn do_something(&self, data: Data) -> Reply {
        let new_str = format!("Do something, answer is: {} modified!", data);
        println!("{}", new_str);

        new_str + " reply!"
    }
}

#[tokio::test]
async fn broke_test() {
    let broker = Arc::new(Delegate::new());
    let _ = ConsumerTest::new(broker.clone()).await;

    let first = broker.broadcast("test_channel", "first".to_string()).await;
    let second = broker.broadcast("test_channel", "second".to_string()).await;

    println!("Result: {} AND {}", first, second);
}
