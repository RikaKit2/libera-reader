use crate::app_dirs::AppDirs;
use crate::db::DB;
use crate::types::LibraryEvent;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::sync::broadcast;

pub async fn run_data_extraction_service(
  db: DB, app_dirs: AppDirs,
  mut high_rx: tokio::sync::mpsc::UnboundedReceiver<crate::db::models::books::book::BookPath>,
  mut low_rx: tokio::sync::mpsc::UnboundedReceiver<crate::db::models::books::book::BookPath>,
  processing_tracker: Arc<
    std::sync::RwLock<crate::types::HashSet<crate::db::models::books::book::BookPath>>,
  >,
  event_tx: broadcast::Sender<LibraryEvent>,
) {
  let available_threads = num_cpus::get().max(2);
  println!("[Extractor] SERVICE STARTING | Threads: {}", available_threads);
  let semaphore = Arc::new(Semaphore::new(available_threads));

  loop {
    // 1. Smart prioritization: first process high-priority queue (items currently visible)
    let path_to_process = if let Ok(path) = high_rx.try_recv() {
      Some(path)
    } else if let Ok(path) = low_rx.try_recv() {
      Some(path)
    } else {
      tokio::time::sleep(Duration::from_millis(100)).await;
      None
    };

    if let Some(path) = path_to_process {
      let permit = Arc::clone(&semaphore).acquire_owned().await.unwrap();
      let db = db.clone();
      let app_dirs = app_dirs.clone();
      let tracker = processing_tracker.clone();
      let event_tx = event_tx.clone();

      tokio::spawn(async move {
        if let Ok(Some(book)) = db.get_book(path.clone()) {
          let (thumbnails_dir, mutool_cmd) = {
            let dirs = app_dirs.read();
            let path = dirs.mutool.clone();
            let cmd =
              if path.exists() { path.to_path_buf() } else { std::path::PathBuf::from("mutool") };
            (dirs.thumbnails_dir.join("unhashed_books"), cmd)
          };

          let path_to_thumbnail =
            thumbnails_dir.join(book.book_path.file_name().as_ref()).with_extension("png");
          let book_full_path = book.book_path.as_pathbuf();

          // 2. If thumbnail file is not created yet
          if !path_to_thumbnail.exists() {
            let _ = tokio::fs::create_dir_all(&thumbnails_dir).await;

            // 3. Call mutool directly with proper thumbnail args
            let output = tokio::process::Command::new(&mutool_cmd)
              .arg("draw")
              .arg("-r")
              .arg("20") // VERY IMPORTANT: low resolution for thumbnail
              .arg("-F")
              .arg("png") // IMPORTANT: PNG format
              .arg("-o")
              .arg(&path_to_thumbnail)
              .arg(&book_full_path)
              .arg("1") // IMPORTANT: only first page
              .stdout(std::process::Stdio::null())
              .stderr(std::process::Stdio::piped())
              .output()
              .await;

            match output {
              Ok(out) if out.status.success() => {
                if path_to_thumbnail.exists() {
                  let _ = event_tx.send(LibraryEvent::BookUpdated(book));
                }
              }
              Ok(out) => {
                eprintln!(
                  "[Extractor] Mutool FAILED for {}. Exit code: {:?}. Stderr: {}",
                  book.book_path.file_name(),
                  out.status.code(),
                  String::from_utf8_lossy(&out.stderr)
                );
              }
              Err(e) => {
                eprintln!(
                  "[Extractor] Failed to execute mutool for {}: {:?}",
                  book.book_path.file_name(),
                  e
                );
              }
            }
          } else {
            // If thumbnail exists, notify UI to update
            let _ = event_tx.send(LibraryEvent::BookUpdated(book));
          }
        }

        // 4. Remove "processing" tag to avoid hangs on failure
        if let Ok(mut t) = tracker.write() {
          t.swap_remove(&path);
        }
        drop(permit);
      });
    }
  }
}
