use crate::app_dirs::AppDirs;
use crate::db::DB;
use crate::db::models::books::book::BookPath;
use crate::settings::SETTINGS;
use crate::types::LibraryEvent;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::{Semaphore, broadcast};

pub async fn run_data_extraction_service(
  db: DB, app_dirs: AppDirs, mut rx: UnboundedReceiver<BookPath>,
  event_tx: broadcast::Sender<LibraryEvent>, settings: SETTINGS,
) {
  let initial_workers = settings.read().workers_num.max(1) as usize;
  println!("[Extractor] SERVICE STARTING | Workers: {}", initial_workers);
  let semaphore = Arc::new(Semaphore::new(initial_workers));
  let max_workers = Arc::new(AtomicUsize::new(initial_workers));

  loop {
    let path = match rx.recv().await {
      Some(p) => p,
      None => break,
    };

    // Re-read workers count and grow semaphore if needed
    let current_workers = settings.read().workers_num.max(1) as usize;
    let prev_max = max_workers.swap(current_workers, Ordering::Relaxed);
    if current_workers > prev_max {
      semaphore.add_permits(current_workers - prev_max);
    }
    // Decreasing is handled naturally — existing tasks finish and permits aren't replenished.

    let permit = Arc::clone(&semaphore).acquire_owned().await.unwrap();
    let db = db.clone();
    let app_dirs = app_dirs.clone();
    let event_tx = event_tx.clone();

    tokio::spawn(async move {
      if let Ok(Some(book)) = db.get_book(path.clone()) {
        let thumbnails_dir = app_dirs.read().thumbnails_dir.join("unhashed_books");
        let path_to_thumbnail =
          thumbnails_dir.join(book.book_path.file_name().as_ref()).with_extension("png");

        let res =
          mutool::extract_img::extract_img(&book.book_path.as_pathbuf(), 20, &path_to_thumbnail)
            .await;

        if res.is_ok() {
          let _ = event_tx.send(LibraryEvent::ThumbnailExtracted(book.book_path.clone()));
        }
      }
      drop(permit);
    });
  }
}
