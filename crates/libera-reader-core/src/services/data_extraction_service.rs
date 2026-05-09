use crate::app_dirs::AppDirs;
use crate::db::DB;
use crate::types::LibraryEvent;
use tokio::sync::broadcast;

/// Simple sequential thumbnail extraction service.
/// Processes books one by one in FIFO order using `mutool::extract_img`.
/// Emits `ThumbnailExtracted` when a thumbnail is ready.
pub async fn run_data_extraction_service(
  db: DB, app_dirs: AppDirs,
  mut rx: tokio::sync::mpsc::UnboundedReceiver<crate::db::models::books::book::BookPath>,
  event_tx: broadcast::Sender<LibraryEvent>,
) {
  println!("[Extractor] SERVICE STARTING | SEQUENTIAL MODE (1 Thread)");

  // Strictly sequential: process one book at a time
  while let Some(path) = rx.recv().await {
    if let Ok(Some(book)) = db.get_book(path.clone()) {
      let thumbnails_dir = app_dirs.read().thumbnails_dir.join("unhashed_books");
      let path_to_thumbnail =
        thumbnails_dir.join(book.book_path.file_name().as_ref()).with_extension("png");

      // Use your own extract_img function!
      // It checks exists() internally and only runs mutool if needed.
      let res =
        mutool::extract_img::extract_img(&book.book_path.as_pathbuf(), 20, &path_to_thumbnail)
          .await;

      if res.is_ok() {
        // Notify UI that thumbnail is ready
        let _ = event_tx.send(LibraryEvent::ThumbnailExtracted(book.book_path.clone()));
      }
    }
  }
}
