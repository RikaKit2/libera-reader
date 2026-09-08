use crate::app_dirs::AppDirs;
use crate::books_state::BooksState;
use crate::db::DB;
use crate::db::models::books::BookType::{self, DuplicateSize};
use crate::db::models::books::DuplicateBookData::{BookHash, MutoolData};
use crate::db::models::books::book::{BookPath, BookSize};
use crate::db::models::books::book_hashes::BookHashes;
use crate::db::models::books::book_sizes::BookSizes;
use crate::services::extraction_coordinator::{ExtractionCoordinator, ExtractionCoordinatorMode};
use crate::settings::SETTINGS;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::Semaphore;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::watch::Receiver;

pub async fn run(
  db: DB, app_dirs: AppDirs, mut rx: UnboundedReceiver<BookPath>, books_state: BooksState,
  settings: SETTINGS, mut coordinator_rx: Receiver<ExtractionCoordinatorMode>,
) {
  let initial_workers = settings.read().workers_num.max(1) as usize;
  let semaphore = Arc::new(Semaphore::new(initial_workers));
  let max_workers = Arc::new(AtomicUsize::new(initial_workers));

  loop {
    // 1. Wait if the user is actively reading a book in BookViewer
    ExtractionCoordinator::wait_for_library_mode(&mut coordinator_rx).await;

    let path = match rx.recv().await {
      Some(p) => p,
      None => break,
    };

    // 2. Re-check coordinator state before acquiring permit
    ExtractionCoordinator::wait_for_library_mode(&mut coordinator_rx).await;

    // 3. Dynamic worker pool scaling based on user settings
    let current_workers = settings.read().workers_num.max(1) as usize;
    let prev_max = max_workers.swap(current_workers, Ordering::Relaxed);
    if current_workers > prev_max {
      semaphore.add_permits(current_workers - prev_max);
    }

    let permit = match Arc::clone(&semaphore).acquire_owned().await {
      Ok(p) => p,
      Err(_) => break,
    };

    let db = db.clone();
    let app_dirs = app_dirs.clone();
    let books_state = books_state.clone();

    tokio::spawn(async move {
      if let Ok(Some(book)) = crate::db::models::books::book::Book::get(&db, path.clone()) {
        // 1. Skip zero-byte empty books
        let BookSize::BYTES(book_size_bytes) = book.book_size;
        if book_size_bytes == 0 {
          drop(permit);
          return;
        }

        // 2. Compute fallback thumbnail path and check if PNG already exists on disk
        let fallback_path = app_dirs.dir_of_unhashed_books.join(format!("{}.png", book_size_bytes));
        if fallback_path.exists() {
          books_state.mark_thumbnail_extracted(&book.book_path);
          drop(permit);
          return;
        }

        // 3. Skip if a usable thumbnail already exists on disk
        if book.has_thumbnail_on_disk(&db, &app_dirs.thumbnails_dir) {
          books_state.mark_thumbnail_extracted(&book.book_path);
          drop(permit);
          return;
        }

        // Determine correct thumbnail path based on book_sizes and book_hashes
        let mut path_to_thumbnail = None;
        let mut computed_hash = None;

        if let Ok(Some(book_sizes)) = db.get_primary::<BookSizes>(book.book_size) {
          match book_sizes.book_type {
            BookType::UniqueSize { .. } => {
              path_to_thumbnail = Some(fallback_path.clone());
            }
            DuplicateSize(map) => {
              if let Some(dup_data) = map.get(&book.book_path) {
                match dup_data {
                  BookHash(hash) => {
                    path_to_thumbnail =
                      Some(app_dirs.dir_of_hashed_books.join(format!("{}.png", hash.0)));
                    computed_hash = Some(hash.clone());
                  }
                  MutoolData(_) => match fallback_path.exists() {
                    true => {
                      path_to_thumbnail = Some(fallback_path.clone());
                    }
                    false => {
                      if let Ok(hash_str) =
                        crate::utils::calc_file_hash(book.book_path.as_pathbuf()).await
                      {
                        let hash = crate::db::models::books::BookHash(hash_str.into());
                        let hashed_path =
                          app_dirs.dir_of_hashed_books.join(format!("{}.png", hash.0));
                        match hashed_path.exists() {
                          true => {
                            path_to_thumbnail = Some(hashed_path);
                          }
                          false => {
                            path_to_thumbnail = Some(hashed_path);
                            computed_hash = Some(hash);
                          }
                        }
                      }
                    }
                  },
                }
              }
            }
          }
        }

        let path_to_thumbnail = path_to_thumbnail.unwrap_or(fallback_path);

        if let Some(parent) = path_to_thumbnail.parent() {
          let _ = std::fs::create_dir_all(parent);
        }

        // 4. Extract PNG to disk if it doesn't exist yet
        let extraction_ok = match path_to_thumbnail.exists() {
          true => true,
          false => {
            let res = mutool::extract_img::extract_img(
              &book.book_path.as_pathbuf(),
              20,
              &path_to_thumbnail,
            )
            .await;

            match res {
              Ok(()) => true,
              Err(err) => {
                let _ = db.rw_t(|rw_t| {
                  if let Some(old_book_sizes) = rw_t.get().primary::<BookSizes>(book.book_size)? {
                    let mut updated_book_sizes = old_book_sizes.clone();
                    match &mut updated_book_sizes.book_type {
                      BookType::UniqueSize { mutool_data, .. } => {
                        let mut md = mutool_data.take().unwrap_or_default();
                        md.mutool_err = Some(err.clone());
                        *mutool_data = Some(md);
                        rw_t.update::<BookSizes>(old_book_sizes, updated_book_sizes)?;
                      }
                      DuplicateSize(map) => {
                        if let Some(dup_data) = map.get_mut(&book.book_path) {
                          match dup_data {
                            MutoolData(m) => {
                              let mut md = m.take().unwrap_or_default();
                              md.mutool_err = Some(err.clone());
                              *m = Some(md);
                              rw_t.update::<BookSizes>(old_book_sizes, updated_book_sizes)?;
                            }
                            BookHash(hash) => {
                              if let Some(old_book_hashes) =
                                rw_t.get().primary::<BookHashes>(hash.clone())?
                              {
                                let mut updated_book_hashes = old_book_hashes.clone();
                                updated_book_hashes.mutool_data.mutool_err = Some(err.clone());
                                rw_t.update::<BookHashes>(old_book_hashes, updated_book_hashes)?;
                              }
                            }
                          }
                        }
                      }
                    }
                  }
                  Ok(())
                });
                false
              }
            }
          }
        };

        if extraction_ok {
          let db_save_res = db.rw_t(|rw_t| {
            if let Some(hash) = &computed_hash
              && let Some(old_book_sizes) = rw_t.get().primary::<BookSizes>(book.book_size)?
            {
              let mut updated_book_sizes = old_book_sizes.clone();
              if let BookType::DuplicateSize(map) = &mut updated_book_sizes.book_type
                && let Some(MutoolData(_)) = map.get(&book.book_path)
              {
                *map.get_mut(&book.book_path).unwrap() = BookHash(hash.clone());
                rw_t.update::<BookSizes>(old_book_sizes, updated_book_sizes)?;

                crate::db::models::books::book_hashes::BookHashes::insert_book(
                  hash.clone(),
                  &book.book_path,
                  rw_t,
                )?;
              }
            }
            Ok(())
          });

          if db_save_res.is_ok() {
            books_state.mark_thumbnail_extracted(&book.book_path);
          }
        }
      }
      drop(permit);
    });
  }
}
