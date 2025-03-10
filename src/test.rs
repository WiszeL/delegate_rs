use std::{fmt::Error, sync::Arc};

use crate::{listens, make_data, make_reply, Data, Delegate, Reply};

struct ConsumerTest;

impl ConsumerTest {
    pub async fn new(delegate: Arc<Delegate<Error>>) -> Arc<Self> {
        let consumer = Arc::new(Self);

        listens!(delegate, consumer, test_channel);

        consumer
    }

    fn test_channel(&self, data: Data) -> Result<Reply, Error> {
        let data = *data.downcast::<String>().unwrap();
        let new_str = format!("Do something, answer is: {} modified!", data);
        println!("{}", new_str);

        make_reply!("".to_string())
    }
}

#[tokio::test]
async fn broke_test() {
    let delegate = Arc::new(Delegate::new());
    let _ = ConsumerTest::new(delegate.clone()).await;

    let first = delegate.broadcast("test_channel", make_data!("first".to_string())).await.unwrap().downcast::<String>().unwrap();
    let second = delegate.broadcast("test_channel", make_data!("second".to_string())).await.unwrap().downcast::<String>().unwrap();

    println!("Result: {} AND {}", first, second);
}
