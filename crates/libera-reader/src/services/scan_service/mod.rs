use crate::{
  app_dirs::AppDirs,
  books_state::BooksState,
  db::{
    DB,
    models::books::book::{Book, BookPath},
  },
  not_cached_books::NotCachedBooks,
  settings::SETTINGS,
  types::MUPDF_EXTENSIONS,
};
use std::{path::PathBuf, time::Instant};

use jwalk::WalkDir;

use crate::types::HashSet;
use crate::utils::debug;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::time::{self, Duration};

#[derive(Clone)]
pub struct ScanService {
  settings: SETTINGS,
  db: DB,
  books_state: BooksState,
  not_cached_books: NotCachedBooks,
  app_dirs: AppDirs,
}

impl ScanService {
  pub fn new(cx: &gpui::App) -> Self {
    use crate::app_ext::AppExt;
    Self::from_deps(
      cx.settings().clone(),
      cx.db().clone(),
      cx.not_cached_books().clone(),
      cx.app_dirs().clone(),
      cx.books_state().clone(),
    )
  }

  pub fn from_deps(
    settings: SETTINGS, db: DB, not_cached_books: NotCachedBooks, app_dirs: AppDirs,
    books_state: BooksState,
  ) -> Self {
    Self { settings, db, books_state, not_cached_books, app_dirs }
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
        // Collect all existing book paths from DB into a HashSet for O(1) lookup
        let db_books = Book::scan_all(&self.db)?;
        let mut existing_paths: HashSet<BookPath> =
          db_books.into_iter().map(|b| b.book_path).collect();

        let rx = Self::get_books_from_disk(path_to_scan);

        self.run_event_loop(rx, &mut existing_paths).await;

        // After scanning, remaining paths are books that no longer exist on disk
        let removed_count = self.remove_outdated_books(existing_paths).await;

        // Send only books without thumbnails to extraction queue in alphabetical order.
        // We re-check the filesystem for each book because `Book` no longer
        // carries a `has_thumbnail` cache field (it could desync from reality).
        let all_books = Book::scan_all(&self.db)?;
        let thumbnails_dir = self.app_dirs.thumbnails_dir.clone();
        let mut books_to_extract: Vec<Book> = all_books
          .into_iter()
          .filter(|b| !b.has_thumbnail_on_disk(&self.db, &thumbnails_dir))
          .collect();
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
    &self, mut rx: UnboundedReceiver<BookPath>, existing_paths: &mut HashSet<BookPath>,
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
        let new_count = self.insert_books(buffer, existing_paths);
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

  fn insert_books(&self, buffer: Vec<BookPath>, existing_paths: &mut HashSet<BookPath>) -> usize {
    let mut new_books = 0;
    let mut inserted_books: Vec<Book> = Vec::new();

    let _ = self.db.rw_t(|rw_t| {
      for book_path in buffer {
        // Mark as found on disk so it is not considered outdated and deleted!
        existing_paths.swap_remove(&book_path);

        // Check if already in DB
        if rw_t.get().primary::<Book>(book_path.clone())?.is_some() {
          continue;
        }
        new_books += 1;
        if let Ok(new_book) = Book::new(book_path) {
          // Insert into BookSizes first
          let _ = crate::db::models::books::book_sizes::BookSizes::insert_book(&new_book, rw_t);
          inserted_books.push(new_book.clone());
          rw_t.insert::<Book>(new_book)?;
        }
      }
      Ok(())
    });

    // Build snapshots outside the transaction so we can consult the filesystem
    // through `has_thumbnail_on_disk` (which needs an `&DB`, not an `RwTransaction`).
    if !inserted_books.is_empty() {
      self.books_state.add_books_batch(&inserted_books);
    }
    new_books
  }

  async fn remove_outdated_books(&self, remaining_paths: HashSet<BookPath>) -> usize {
    let mut removed_count = 0;

    // Collect updated books inside the transaction; build snapshots afterwards so
    // we can call `has_thumbnail_on_disk` with an `&DB` reference.
    let mut updated_books: Vec<Book> = Vec::new();
    let mut removed_paths: Vec<BookPath> = Vec::new();

    let _ = self.db.rw_t(|rw_t| {
      for book_path in remaining_paths {
        if let Some(book) = rw_t.get().primary::<Book>(book_path.clone())? {
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
            removed_paths.push(book_path);
          } else {
            let mut updated_book = book.clone();
            updated_book.mark_as_deleted();
            updated_books.push(updated_book.clone());
            rw_t.update::<Book>(book, updated_book)?;
          }
          removed_count += 1;
        }
      }
      Ok(())
    });

    self.books_state.remove_books(&removed_paths);
    for updated_book in updated_books {
      self.books_state.update_book(&updated_book);
    }
    removed_count
  }
}
