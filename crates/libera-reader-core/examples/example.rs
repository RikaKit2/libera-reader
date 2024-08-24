use std::io;
use std::time::Duration;

use libera_reader_core::app_core::{AppCore, AppCoreError};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
  let subscriber = tracing_subscriber::fmt()
    .pretty()
    .without_time()
    // Use a more compact, abbreviated log format
    .compact()
    // Display source code file paths
    .with_file(true)
    // Display source code line numbers
    .with_line_number(true)
    // Display the thread ID an event was recorded on
    .with_thread_ids(true)
    // Don't display the event's target (module path)
    .with_target(false)
    // Build the subscriber
    .finish();
  tracing::subscriber::set_global_default(subscriber).unwrap();
  match AppCore::new(Option::from(None)).await {
    Ok(mut app_core) => {
      match app_core.settings.get_path_to_scan().await {
        Some(_path_to_scan) => {
          app_core.services.run().await;
        }
        None => {
          println!(" Please input path to scan:");
          let mut user_input = String::new();
          io::stdin().read_line(&mut user_input).expect("error: unable to read user input");
          let user_input= user_input.trim().to_string();
          app_core.settings.set_path_to_scan(user_input).await;
          app_core.services.run().await;
        }
      };
      loop {
        tokio::time::sleep(Duration::from_secs(10)).await;
      }
    }
    Err(err) => {
      match err {
        AppCoreError::DirsCreationErr(e) => {
          for i in e {
            println!("{:?}", i.to_string());
          }
        }
        AppCoreError::PrismaErr(e) => {
          println!("{:?}", e.to_string());
        }
      }
    }
  };
}

