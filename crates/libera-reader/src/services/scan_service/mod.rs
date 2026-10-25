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
    let Some(path_to_scan) = self.settings.get_path_to_scan_if_exists() else {
      debug!("Path to scan is not set. Please set it in the settings.");
      return Ok(());
    };

    let db = self.db.clone();
    let existing_paths: HashSet<BookPath> =
      tokio::task::spawn_blocking(move || -> anyhow::Result<HashSet<BookPath>> {
        let db_books = Book::scan_all(&db)?;
        Ok(db_books.into_iter().map(|b| b.book_path).collect())
      })
      .await??;

    let rx = Self::get_books_from_disk(path_to_scan);
    let remaining_paths = self.run_event_loop(rx, existing_paths).await;

    let removed_count = self.remove_outdated_books(remaining_paths).await;

    let db = self.db.clone();
    let thumbnails_dir = self.app_dirs.thumbnails_dir.clone();
    let books_to_extract = tokio::task::spawn_blocking(move || -> anyhow::Result<Vec<BookPath>> {
      let all_books = Book::scan_all(&db)?;
      let mut missing: Vec<Book> = all_books
        .into_iter()
        .filter(|b| !b.book_path.deleted && !b.has_thumbnail_on_disk(&db, &thumbnails_dir))
        .collect();
      missing.sort_by_key(|b| b.book_path.display_name().to_lowercase());
      Ok(missing.into_iter().map(|b| b.book_path).collect())
    })
    .await??;

    let tx = self.not_cached_books.tx();
    for book_path in books_to_extract {
      let _ = tx.send(book_path);
    }

    debug!("Full scanning process finished. Removed {} outdated books.", removed_count);
    Ok(())
  }

  async fn run_event_loop(
    &self, mut rx: UnboundedReceiver<BookPath>, mut existing_paths: HashSet<BookPath>,
  ) -> HashSet<BookPath> {
    let mut total_new_books = 0;
    let total_insert_start = Instant::now();

    loop {
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
        for bp in &buffer {
          existing_paths.swap_remove(bp);
        }
        let this = self.clone();
        let new_count =
          tokio::task::spawn_blocking(move || this.insert_books(buffer)).await.unwrap_or(0);
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
    existing_paths
  }

  fn insert_books(&self, buffer: Vec<BookPath>) -> usize {
    let mut new_books = 0;
    let mut inserted_books: Vec<Book> = Vec::new();

    let _ = self.db.rw_t(|rw_t| {
      for book_path in buffer {
        if let Some(existing_book) = rw_t.get().primary::<Book>(book_path.clone())? {
          if existing_book.book_path.deleted {
            let mut restored = existing_book.clone();
            restored.book_path.deleted = false;
            let _ = crate::db::models::books::book_sizes::BookSizes::insert_book(&restored, rw_t);
            rw_t.update::<Book>(existing_book, restored.clone())?;
            inserted_books.push(restored);
            new_books += 1;
          }
          continue;
        }
        new_books += 1;
        if let Ok(new_book) = Book::new(book_path) {
          if let Ok(Some(_prev_unique_path)) =
            crate::db::models::books::book_sizes::BookSizes::insert_book(&new_book, rw_t)
          {
            let crate::db::models::books::book::BookSize::BYTES(size) = new_book.book_size;
            let unhashed_dir = self.app_dirs.book_dir(size, None);
            if unhashed_dir.exists() {
              let _ = std::fs::remove_dir_all(&unhashed_dir);
            }
            let legacy_path = self.app_dirs.legacy_book_cover_path(size, None);
            if legacy_path.exists() {
              let _ = std::fs::remove_file(&legacy_path);
            }
          }
          inserted_books.push(new_book.clone());
          rw_t.insert::<Book>(new_book)?;
        }
      }
      Ok(())
    });

    if !inserted_books.is_empty() {
      self.books_state.add_books_batch(&inserted_books);
    }
    new_books
  }

  async fn remove_outdated_books(&self, remaining_paths: HashSet<BookPath>) -> usize {
    if remaining_paths.is_empty() {
      return 0;
    }
    let this = self.clone();
    tokio::task::spawn_blocking(move || {
      let mut removed_count = 0;
      let mut updated_books: Vec<Book> = Vec::new();
      let mut removed_paths: Vec<BookPath> = Vec::new();

      let _ = this.db.rw_t(|rw_t| {
        for book_path in remaining_paths {
          if let Some(book) = rw_t.get().primary::<Book>(book_path.clone())? {
            if book.book_path.deleted {
              continue;
            }
            let can_delete = book.can_delete();
            let book_path = book.book_path.clone();

            if can_delete {
              let crate::db::models::books::book::BookSize::BYTES(size) = book.book_size;
              let hash = if let Ok(Some(book_sizes)) =
                rw_t
                  .get()
                  .primary::<crate::db::models::books::book_sizes::BookSizes>(book.book_size)
              {
                match book_sizes.book_type {
                  crate::db::models::books::BookType::DuplicateSize(map) => {
                    if let Some(crate::db::models::books::DuplicateBookData::BookHash(h)) =
                      map.get(&book.book_path)
                    {
                      Some(h.0.to_string())
                    } else {
                      None
                    }
                  }
                  _ => None,
                }
              } else {
                None
              };

              let book_cache_dir = this.app_dirs.book_dir(size, hash.as_deref());
              if book_cache_dir.exists() {
                let _ = std::fs::remove_dir_all(&book_cache_dir);
              }
              let legacy_path = this.app_dirs.legacy_book_cover_path(size, hash.as_deref());
              if legacy_path.exists() {
                let _ = std::fs::remove_file(&legacy_path);
              }

              let _ = crate::db::models::books::book_sizes::BookSizes::remove_book(
                book.book_size,
                &book.book_path,
                rw_t,
              );
              if let Some(bookmarks) =
                rw_t.get().primary::<crate::db::models::books::bookmark::BookBookmarks>(
                  book.book_path.clone(),
                )?
              {
                rw_t.remove::<crate::db::models::books::bookmark::BookBookmarks>(bookmarks)?;
              }
              rw_t.remove::<Book>(book)?;
              removed_paths.push(book_path);
            } else {
              let _ = crate::db::models::books::book_sizes::BookSizes::mark_book_path_as_deleted(
                book.book_size,
                &book.book_path,
                rw_t,
              );
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

      this.books_state.remove_books(&removed_paths);
      for updated_book in updated_books {
        this.books_state.update_book(&updated_book);
      }
      removed_count
    })
    .await
    .unwrap_or(0)
  }
}
