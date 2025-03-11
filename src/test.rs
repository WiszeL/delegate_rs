use std::{fmt::Error, sync::Arc};

use crate::{listens, Delegate};

struct ConsumerTest;

impl ConsumerTest {
    pub fn new(delegate: Arc<Delegate<Error>>) -> Arc<Self> {
        let consumer = Arc::new(Self);

        listens!(delegate, consumer, test_channel);

        consumer
    }

    fn test_channel(&self, data: String) -> Result<String, Error> {
        let new_str = format!("Do something, answer is: {} modified!", data);
        println!("{}", new_str);

        Ok("".to_string())
    }
}

#[test]
fn broke_test() {
    let delegate = Arc::new(Delegate::new());
    let _ = ConsumerTest::new(delegate.clone());

    let first: String = delegate.broadcast("test_channel", "first".to_string()).unwrap();
    let second: String = delegate.broadcast("test_channel", "second".to_string()).unwrap();

    println!("Result: {} AND {}", first, second);
}
