use crate::services::{Services, Status::{NotWorking, Working}};
use crate::types::{NotCachedBooks, APP_DIRS, DB};
use mutool_bindings::extract_img::extract_img;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;
use tracing::{error, info};

impl Services {
  pub async fn run_data_extraction_service(&mut self) -> Option<JoinHandle<()>> {
    match &self.data_extraction_service_working_status {
      Working => { None }
      NotWorking => {
        let db = self.db.clone();
        let not_cached_books = self.not_cached_books.clone();
        let app_dirs = self.app_dirs.clone();
        let j = tokio::spawn(async move {
          run(db, not_cached_books, app_dirs).await;
        });
        self.data_extraction_service_working_status = Working;
        info!("Data extraction service started");
        Some(j)
      }
    }
  }
}

async fn run(db: DB, not_cached_books: NotCachedBooks, app_dirs: APP_DIRS) {
  let available_threads: usize = num_cpus::get();
  info!("Num of threads for data_extraction_service: {}", &available_threads);
  let semaphore = Arc::new(Semaphore::new(available_threads));

  loop {
    if not_cached_books.is_empty() == false {
      info!("Num of books for caching: {}", not_cached_books.len());

      let start_time = std::time::Instant::now();
      let mut tasks = Vec::new();

      for book in not_cached_books.try_iter() {
        let semaphore = Arc::clone(&semaphore);
        let app_dirs = app_dirs.clone();
        let db = db.clone();

        let task = tokio::spawn(async move {
          let permit = semaphore.acquire().await.unwrap();
          let path_to_thumbnail = book.path_to_thumbnail_for_mutool(&app_dirs);
          let path = book.full_path.clone();
          match extract_img(&PathBuf::from(&book.full_path), 20, &path_to_thumbnail).await {
            Ok(_) => {
              match book.mark_as_cached(&db) {
                Ok(_) => {}
                Err(err) => {
                  error!("Error while marking book as cached: {}\npath: {}", err.to_string(), &path);
                }
              }
            }
            Err(mutool_err) => {
              eprint!("{:?}", &mutool_err);
              match book.mark_as_broken(mutool_err, &db) {
                Ok(_) => {}
                Err(err) => {
                  error!("Error while marking book as broken: {}\npath: {}", err.to_string(), &path);
                }
              }
            }
          }
          drop(permit);
        });

        tasks.push(task);
      }

      for task in tasks {
        let _ = task.await;
      }

      info!("Total time of thumbnails extracting: {:.2?}", start_time.elapsed());
    }
    tokio::time::sleep(Duration::from_secs(1)).await;
  }
}
