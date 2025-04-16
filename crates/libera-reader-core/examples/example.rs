use libera_reader_core::{services::Services, vars::SETTINGS};
use std::io;
use std::thread::sleep;
use std::time::Duration;
use tracing::Level;


fn main() {
  let subscriber = tracing_subscriber::fmt()
    .pretty()
    .without_time()
    .compact()
    .with_file(false)
    .with_line_number(false)
    .with_thread_ids(true)
    .with_target(false)
    .with_max_level(Level::DEBUG)
    .finish();
  tracing::subscriber::set_global_default(subscriber).unwrap();

  let mut services = Services::new();
  match SETTINGS.read().unwrap().path_to_scan.is_some() {
    true => { services.run(); }
    false => {
      println!("Please input path to scan:");
      let mut user_input = String::new();
      io::stdin().read_line(&mut user_input).expect("Error: unable to read user input");
      let user_input = user_input.trim().to_string();
      SETTINGS.write().unwrap().set_path_to_scan(user_input);
      services.run();
    }
  };
  loop {
    sleep(Duration::from_secs(10));
  }
}
