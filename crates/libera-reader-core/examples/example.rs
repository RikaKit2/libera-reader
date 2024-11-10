use libera_reader_core::app_core::AppCore;
use std::io;
use std::time::Duration;
use tracing::error;


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
  match AppCore::new().await {
    Ok(mut app_core) => {
      match app_core.settings.path_to_scan_is_valid().await {
        true => {
          app_core.services.run().await;
        }
        false => {
          println!("Please input path to scan:");
          let mut user_input = String::new();
          io::stdin().read_line(&mut user_input).expect("Error: unable to read user input");
          let user_input = user_input.trim().to_string();
          app_core.settings.set_path_to_scan(user_input).await;
          app_core.services.run().await;
        }
      };
      loop {
        tokio::time::sleep(Duration::from_secs(10)).await;
        println!("Example of an infinite loop")
      }
    }
    Err(poss_errors) => {
      poss_errors.iter().for_each(|e| error!("{}", e));
    }
  };
}
