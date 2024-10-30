use measure_time_macro::measure_time;
use std::time::Duration;
use tokio;
use tracing::{event, Level};


#[measure_time]
async fn some_async_fn(x: i32) -> i32 {
  tokio::time::sleep(Duration::from_secs(1)).await;
  x * 2
}

#[measure_time]
fn some_sync_fn(x: i32) -> i32 {
  std::thread::sleep(Duration::from_secs(1));
  x * 2
}


#[tokio::main]
async fn main() {
  let subscriber = tracing_subscriber::fmt().pretty()
    .without_time()
    .compact()
    .with_file(false)
    .with_line_number(false)
    .with_thread_ids(false)
    .with_target(false)
    .finish();
  tracing::subscriber::set_global_default(subscriber).unwrap();
  let async_result = some_async_fn(5).await;
  println!("Async Result: {}", async_result);

  let sync_result = some_sync_fn(5);
  println!("Sync Result: {}", sync_result);
}
