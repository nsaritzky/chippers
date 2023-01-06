use tokio::{task, time};
use std::time::Duration;
use futures::{stream};


#[tokio::main]
pub async fn main() {
    let forever = task::spawn(async {
        let mut interval = time::interval(Duration::from_micros(16667));

        loop {
            interval.tick().await;
            
        }
    })
}

async fn timer() {
    let mut timer_val: u8
}
