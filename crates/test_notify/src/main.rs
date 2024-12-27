use notify::{RecursiveMode, Watcher};
use std::io;
use std::path::PathBuf;
use std::time::Duration;
use tracing::error;

mod notify_service;


#[tokio::main(flavor = "multi_thread")]
async fn main() {
  let (tx, notify_receiver) = crossbeam_channel::unbounded();
  let mut watcher = notify::recommended_watcher(move |res| { tx.send(res).unwrap(); }).unwrap();
  println!("Please input path to scan:");
  let mut user_input = String::new();
  io::stdin().read_line(&mut user_input).expect("Error: unable to read user input");
  let user_input = user_input.trim();
  match PathBuf::from(user_input).is_dir() {
    true => {
      watcher.watch(user_input.as_ref(), RecursiveMode::Recursive).unwrap();
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
      match tokio::spawn(notify_service::run(notify_receiver)).await {
        Ok(_) => {}
        Err(err) => {
          error!("{:?}", err.to_string())
        }
      }
    }
    false => {}
  }
  loop {
    tokio::time::sleep(Duration::from_secs(10)).await;
    println!("Example of an infinite loop")
  }
}


