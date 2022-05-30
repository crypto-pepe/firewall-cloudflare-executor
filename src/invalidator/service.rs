use std::time::Duration;
use tokio::{task, time};

pub struct Invalidator {}

impl Invalidator {
    pub fn new() -> Self {
        return Self {};
    }
    pub async fn run(self) -> Result<(), tokio::task::JoinError> {
        let forever = task::spawn(async {
            let mut interval = time::interval(Duration::from_millis(10));
            loop {
                interval.tick().await;
                do_something().await;
            }
        });
        forever.await
    }
    pub async fn run_invalidator_untill_stopped(self) -> Result<(), tokio::task::JoinError> {
        self.run().await
    }
}

async fn do_something() {
    eprintln!("do_something");
}
