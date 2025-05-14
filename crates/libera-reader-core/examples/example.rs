use concurrent_queue::ConcurrentQueue;
use libera_reader_core::app_dirs::AppDirs;
use libera_reader_core::db::{create_db_in_memory, models::{Settings, TargetExt}};
use libera_reader_core::services::Services;
use libera_reader_core::types::NotCachedBooks;
use std::io;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
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
  let db = Arc::new(create_db_in_memory());
  let app_dirs = Arc::new(RwLock::new(AppDirs::new(PathBuf::from("app_tmp_dir")).unwrap()));
  let target_ext = Arc::new(RwLock::new(TargetExt::new(&db)));
  let not_cached_books = NotCachedBooks::new(ConcurrentQueue::unbounded());
  let mut services = Services::new(target_ext.clone(), app_dirs.clone(), db.clone(), not_cached_books.clone());
  Settings::create_if_not_exist(&db);

  match Settings::get_path_to_scan_db_only(&db) {
    Some(path_to_scan) => { services.run(path_to_scan); }
    None => {
      println!("Please input path to scan:");
      let mut user_input = String::new();
      io::stdin().read_line(&mut user_input).expect("Error: unable to read user input");
      let user_input = user_input.trim().to_string();
      Settings::set_path_to_scan_db_only(user_input.clone(), None, &db);
      services.run(user_input);
    }
  };
  loop {
    sleep(Duration::from_secs(10));
  }
}
