use libera_reader_core::vars::{SERVICES, SETTINGS};
use std::io;
use std::time::Duration;
use tracing::info;


#[tokio::main(flavor = "multi_thread")]
async fn main() {
  let subscriber = tracing_subscriber::fmt()
    .pretty()
    .without_time()
    .compact()
    .with_file(false)
    .with_line_number(false)
    .with_thread_ids(true)
    .with_target(false)
    .finish();
  tracing::subscriber::set_global_default(subscriber).unwrap();
  match SETTINGS.read().unwrap().path_to_scan_is_valid() {
    true => {
      SERVICES.write().unwrap().run().await;
    }
    false => {
      println!("Please input path to scan:");
      let mut user_input = String::new();
      io::stdin().read_line(&mut user_input).expect("Error: unable to read user input");
      let user_input = user_input.trim().to_string();
      info!("0");
      SETTINGS.write().unwrap().set_path_to_scan(user_input);
      info!("1");
      SERVICES.write().unwrap().run().await;
      info!("2");
    }
  };
  loop {
    tokio::time::sleep(Duration::from_secs(10)).await;
    println!("Example of an infinite loop")
  }
}
