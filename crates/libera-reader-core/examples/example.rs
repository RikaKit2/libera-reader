use libera_reader_core::core::Core;
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

  let mut app_core = Core::new();
  match app_core.settings.path_to_scan_is_valid() {
    true => {
      app_core.services.run();
    }
    false => {
      println!("Please input path to scan:");
      let mut user_input = String::new();
      io::stdin().read_line(&mut user_input).expect("Error: unable to read user input");
      let user_input = user_input.trim().to_string();
      app_core.settings.set_path_to_scan(user_input);
      app_core.services.run();
    }
  };
  loop {
    sleep(Duration::from_secs(10));
  }
}
