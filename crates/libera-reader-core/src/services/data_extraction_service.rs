use crate::services::{State, Status::{NotWorking, Working}};
use crate::types::{NotCachedBooks, APP_DIRS, DB};
use mutool_bindings::{extract_img, MuToolResult};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{error, info};

impl State {
  pub fn run_data_extraction_service(&mut self) {
    match &self.data_extraction_service_working_status {
      Working => {}
      NotWorking => {
        tokio::spawn(run_thread(self.db.clone(), self.not_cached_books.clone(), self.app_dirs.clone()));
        self.data_extraction_service_working_status = Working;
      }
    }
  }
}
async fn run_thread(db: DB, not_cached_books: NotCachedBooks, app_dirs: APP_DIRS) {
  let available_threads: usize = num_cpus::get();
  info!("Number of threads: {available_threads}");
  let semaphore = Arc::new(Semaphore::new(available_threads));

  loop {
    match not_cached_books.is_empty() {
      true => {}
      false => {
        let book = not_cached_books.pop().unwrap();
        info!("{:?}", &book.full_path);
        let permit = match semaphore.clone().acquire_owned().await {
          Ok(p) => p,
          Err(_) => break,
        };

        let app_dirs = app_dirs.clone();
        let db = db.clone();
        tokio::spawn(async move {
          let path_to_thumbnail = book.path_to_thumbnail(&app_dirs);
          match extract_img(&PathBuf::from(&book.full_path), 20, &path_to_thumbnail).await {
            Ok(status) => {
              match status {
                MuToolResult::Success => { book.mark_as_cached(&db).unwrap(); }
                _ => { book.mark_as_broken(status, &db).unwrap(); }
              }
            }
            Err(e) => { error!("mutool failed: {}", e); }
          }
          drop(permit);
        });
      }
    }
  }
}
