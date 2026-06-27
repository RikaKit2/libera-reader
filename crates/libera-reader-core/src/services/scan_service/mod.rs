use crate::{
  db::{
    DB,
    models::books::book::{Book, BookPath},
  },
  not_cached_books::NotCachedBooks,
  settings::SETTINGS,
  types::{LibraryEvent, MUPDF_EXTENSIONS},
};
use std::{path::PathBuf, time::Instant};

use jwalk::WalkDir;

use crate::types::HashSet;
use tokio::sync::{broadcast, mpsc::UnboundedReceiver};
use tokio::time::{self, Duration};
use utils::debug;

#[derive(Clone)]
pub struct ScanService {
  settings: SETTINGS,
  db: DB,
  event_tx: broadcast::Sender<LibraryEvent>,
  not_cached_books: NotCachedBooks,
}

impl ScanService {
  pub(crate) fn new(
    settings: SETTINGS, db: DB, event_tx: broadcast::Sender<LibraryEvent>,
    not_cached_books: NotCachedBooks,
  ) -> Self {
    Self { settings, db, event_tx, not_cached_books }
  }

  pub fn settings(&self) -> &SETTINGS {
    &self.settings
  }

  fn get_books_from_disk(path_to_scan: PathBuf) -> UnboundedReceiver<BookPath> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    tokio::task::spawn_blocking(move || {
      let start_time = Instant::now();
      for entry in WalkDir::new(path_to_scan) {
        if let Ok(entry) = entry
          && entry.file_type().is_file()
        {
          let path = entry.path();
          if let Some(ext) = path.extension().and_then(|e| e.to_str())
            && MUPDF_EXTENSIONS.contains(&ext.to_lowercase().as_str())
            && let Some(file_path) = BookPath::new(&path)
            && tx.send(file_path).is_err()
          {
            break;
          }
        }
      }
      debug!("\nWalkDir finished in: {:?}", start_time.elapsed());
    });
    rx
  }

  pub async fn run(&self) -> anyhow::Result<()> {
    match self.settings.get_path_to_scan_if_exists() {
      None => {
        debug!("Path to scan is not set. Please set it in the settings.");
      }
      Some(path_to_scan) => {
        // Collect all existing book IDs from DB into a HashSet for O(1) lookup
        let db_books = self.db.scan_all_books()?;
        let mut existing_ids: HashSet<String> = db_books.iter().map(|b| b.id.clone()).collect();

        let rx = Self::get_books_from_disk(path_to_scan);

        self.run_event_loop(rx, &mut existing_ids).await;

        // After scanning, remaining IDs are books that no longer exist on disk
        let removed_count = self.remove_outdated_books(existing_ids).await;

        // Send only books without thumbnails to extraction queue in alphabetical order
        let all_books = self.db.scan_all_books()?;
        let mut books_to_extract: Vec<Book> =
          all_books.into_iter().filter(|b| !b.has_thumbnail).collect();
        books_to_extract.sort_by_key(|b| b.book_path.display_name().to_lowercase());

        let tx = self.not_cached_books.tx();
        for book in books_to_extract {
          let _ = tx.send(book.book_path);
        }

        debug!("Full scanning process finished. Removed {} outdated books.", removed_count);
      }
    };

    Ok(())
  }

  async fn run_event_loop(
    &self, mut rx: UnboundedReceiver<BookPath>, existing_ids: &mut HashSet<String>,
  ) {
    let mut total_new_books = 0;
    let total_insert_start = Instant::now();

    loop {
      // 1. Wait for the FIRST batch item.
      let first_item = match rx.recv().await {
        Some(item) => item,
        None => break,
      };

      let mut buffer = Vec::with_capacity(500);
      buffer.push(first_item);

      let sleep = time::sleep(Duration::from_millis(300));
      tokio::pin!(sleep);

      loop {
        tokio::select! {
          book_path = rx.recv() => {
              match book_path {
                  Some(item) => {
                      buffer.push(item);
                      if buffer.len() >= 500 {
                          break;
                      }
                  }
                  None => break,
              }
          }
          _ = &mut sleep => break,
        }
      }

      if !buffer.is_empty() {
        let new_count = self.insert_books(buffer, existing_ids);
        total_new_books += new_count;
        eprint!(
          "\rAdding books: {} (elapsed: {:?})",
          total_new_books,
          total_insert_start.elapsed()
        );
      }
    }

    let total_insert_elapsed = total_insert_start.elapsed();
    eprintln!();

    if total_new_books > 0 {
      debug!("Scan complete. Added {} new books in {:?}.", total_new_books, total_insert_elapsed);
    }
  }

  fn insert_books(&self, buffer: Vec<BookPath>, existing_ids: &mut HashSet<String>) -> usize {
    let mut new_books = 0;
    let mut batch_to_send = Vec::new();
    let event_tx = &self.event_tx;

    let _ = self.db.rw_t(|rw_t| {
      for book_path in buffer {
        let id = book_path.full_path_string().to_string();

        // Mark as found on disk so it is not considered outdated and deleted!
        existing_ids.swap_remove(&id);

        // Check if already in DB
        if rw_t.get().primary::<Book>(id.clone())?.is_some() {
          continue;
        }

        new_books += 1;
        if let Ok(new_book) = Book::new(book_path) {
          // Insert into BookSizes first
          let _ = crate::db::models::books::book_sizes::BookSizes::insert_book(&new_book, rw_t);
          rw_t.insert::<Book>(new_book.clone())?;
          batch_to_send.push(new_book);
        }
      }
      Ok(())
    });

    if !batch_to_send.is_empty() {
      let _ = event_tx.send(LibraryEvent::BooksBatchAdded(batch_to_send));
    }

    new_books
  }

  async fn remove_outdated_books(&self, remaining_ids: HashSet<String>) -> usize {
    let mut removed_count = 0;
    let event_tx = &self.event_tx;

    let _ = self.db.rw_t(|rw_t| {
      for id in remaining_ids {
        if let Some(book) = rw_t.get().primary::<Book>(id.clone())? {
          let can_delete = book.can_delete();
          let book_path = book.book_path.clone();

          // Remove from BookSizes
          let _ = crate::db::models::books::book_sizes::BookSizes::remove_book(
            book.book_size,
            &book.book_path,
            rw_t,
          );

          if can_delete {
            rw_t.remove::<Book>(book)?;
            let _ = event_tx.send(LibraryEvent::BookRemoved(book_path));
          } else {
            let mut updated_book = book.clone();
            updated_book.mark_as_deleted();
            let updated_book_clone = updated_book.clone();
            rw_t.update::<Book>(book, updated_book)?;
            let _ = event_tx.send(LibraryEvent::BookUpdated(updated_book_clone));
          }
          removed_count += 1;
        }
      }
      Ok(())
    });

    removed_count
  }
}
