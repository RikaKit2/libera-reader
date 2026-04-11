use crate::{
  db::models::books::{Books, book::Book},
  types::{LibraryEvent, MUPDF_EXTENSIONS},
};
use std::{path::PathBuf, time::Instant};

use jwalk::WalkDir;

use crate::types::HashMap;
use crate::{
  db::{
    DB,
    models::books::book::{BookDir, BookPath},
  },
  settings::SETTINGS,
};
use tokio::sync::{broadcast, mpsc::UnboundedReceiver};
use tokio::time::{self, Duration};
use utils::debug;

pub(crate) type DBBooksCount = usize;
pub(crate) type BooksFromDB = HashMap<BookDir, Books>;

pub struct ScanService {
  settings: SETTINGS,
  db: DB,
  event_tx: broadcast::Sender<LibraryEvent>,
}

impl ScanService {
  pub(crate) fn new(settings: SETTINGS, db: DB, event_tx: broadcast::Sender<LibraryEvent>) -> Self {
    Self { settings, db, event_tx }
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
          {
            // If the channel is closed (UI cancelled scan), tx.send will return an error,
            // and we can safely exit.
            if tx.send(file_path).is_err() {
              break;
            }
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
        let (db_books, _) = self.db.rt(|r| Ok(Books::all(r)))?;

        let rx = Self::get_books_from_disk(path_to_scan);

        self.run_event_loop(rx, db_books).await;
      }
    };

    Ok(())
  }

  async fn run_event_loop(&self, mut rx: UnboundedReceiver<BookPath>, mut db_books: HashMap<BookDir, Books>) {
    let mut total_new_books = 0;
    let total_insert_start = Instant::now();

    loop {
      // 1. Wait for the FIRST batch item.
      // If the channel is empty and closed - exit the entire event loop.
      let first_item = match rx.recv().await {
        Some(item) => item,
        None => break, // Scanning fully completed
      };

      let mut buffer = Vec::with_capacity(500);
      buffer.push(first_item);

      // 2. Once we got the first one, start a 300ms timer to collect the rest
      let sleep = time::sleep(Duration::from_millis(300));
      tokio::pin!(sleep);

      loop {
        tokio::select! {
          book_path = rx.recv() => {
              match book_path {
                  Some(item) => {
                      buffer.push(item);
                      // Memory overflow protection if SSD is too fast
                      if buffer.len() >= 500 {
                          break;
                      }
                  }
                  None => {
                      // Channel closed during batch collection
                      break;
                  }
              }
          }
          _ = &mut sleep => {
              // 300ms timeout elapsed, time to process what we collected
              break;
          }
        }
      }

      if !buffer.is_empty() {
        let new_count = self.insert_books(buffer, &mut db_books);
        total_new_books += new_count;
        eprint!("\rAdding books: {} (elapsed: {:?})", total_new_books, total_insert_start.elapsed());
      }
    }

    let total_insert_elapsed = total_insert_start.elapsed();
    eprintln!();

    let total_remove_start = Instant::now();
    let removed_count = self.remove_outdated_books(db_books).await;
    let total_remove_elapsed = total_remove_start.elapsed();

    if total_new_books > 0 || removed_count > 0 {
      debug!(
        "Scan complete. Added {} new books in {:?}, removed {} outdated books in {:?}.",
        total_new_books, total_insert_elapsed, removed_count, total_remove_elapsed
      );
    }

    debug!("Full scanning process finished.");
  }

  fn insert_books(&self, buffer: Vec<BookPath>, db_books: &mut HashMap<BookDir, Books>) -> usize {
    let mut new_books = 0;
    let event_tx = &self.event_tx;

    let _ = self.db.rw_t(|rw_t| {
      for book_path in buffer {
        match db_books.get_mut(&book_path.parent_dir) {
          Some(dir_books) => match dir_books.storage.swap_remove(&book_path.name) {
            Some(_existing_book) => {}
            None => {
              new_books += 1;
              if let Ok(new_book) = Book::new(book_path.clone()) {
                let _ = Books::insert_book(new_book.clone(), rw_t);
                // Шлём событие для каждой новой книги
                let _ = event_tx.send(LibraryEvent::BookAdded(new_book));
              }
            }
          },

          None => {
            new_books += 1;
            if let Ok(new_book) = Book::new(book_path.clone()) {
              let _ = Books::insert_book(new_book.clone(), rw_t);
              // Шлём событие для каждой новой книги
              let _ = event_tx.send(LibraryEvent::BookAdded(new_book));
            }
          }
        }
      }
      Ok(())
    });

    new_books
  }

  async fn remove_outdated_books(&self, dead_books: HashMap<BookDir, Books>) -> usize {
    let mut outdated_books_count = 0;

    let _ = self.db.rw_t(|rw_t| {
      for (dir, books_in_dir) in dead_books {
        let paths_to_delete: Vec<BookPath> =
          books_in_dir.storage.values().map(|book| book.book_path.clone()).collect();

        for path in paths_to_delete {
          if let Ok(Some(fresh_dir_books)) = Books::get_by_parent_dir_rw(dir.clone(), rw_t) {
            let _ = fresh_dir_books.remove_book(path, rw_t);
            outdated_books_count += 1;
          }
        }
      }
      Ok(())
    });

    outdated_books_count
  }
}
