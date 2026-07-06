use crate::app_dirs::AppDirs;
use crate::db::DB;
use crate::db::models::books::BookType::{self, DuplicateSize};
use crate::db::models::books::DuplicateBookData::{BookHash, MutoolData};
use crate::db::models::books::book::{Book, BookPath, BookSize};
use crate::db::models::books::book_hashes::BookHashes;
use crate::db::models::books::book_sizes::BookSizes;
use crate::send_event;
use crate::settings::SETTINGS;
use crate::types::{HashSet, LibraryEvent};
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

    let permit = Arc::clone(&semaphore).acquire_owned().await.unwrap();
    let db = db.clone();
    let app_dirs = app_dirs.clone();
    let event_tx = event_tx.clone();

    tokio::spawn(async move {
      if let Ok(Some(book)) = db.get_book(path.clone()) {
        // 1. Check if book size is 0 bytes
        let BookSize::BYTES(book_size_bytes) = book.book_size;
        if book_size_bytes == 0 {
          drop(permit);
          return;
        }

        // 2. Compute fallback thumbnail path and check if PNG already exists on disk
        // This handles the case where DB was deleted but PNG files are still on disk.
        let fallback_path = app_dirs
          .read()
          .thumbnails_dir
          .join("unhashed_books")
          .join(format!("{}.png", book_size_bytes));
        if fallback_path.exists() {
          // PNG already on disk — update DB and skip
          let _ = db.rw_t(|rw_t| {
            if let Some(mut b) = rw_t.get().primary::<Book>(book.id.clone())? {
              let old_b = b.clone();
              b.has_thumbnail = true;
              rw_t.update::<Book>(old_b, b)?;
            }
            Ok(())
          });
          send_event!(event_tx, LibraryEvent::ThumbnailExtracted(book.book_path.clone()));
          drop(permit);
          return;
        }

        // 3. Check if thumbnail is already known to be extracted (DB flag)
        if book.has_thumbnail {
          send_event!(event_tx, LibraryEvent::ThumbnailExtracted(book.book_path.clone()));
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
                    path_to_thumbnail = Some(
                      app_dirs
                        .read()
                        .thumbnails_dir
                        .join("hashed_books")
                        .join(format!("{}.png", hash.0)),
                    );
                    computed_hash = Some(hash.clone());
                  }
                  MutoolData(_) => {
                    // First check if fallback exists before computing hash
                    if fallback_path.exists() {
                      path_to_thumbnail = Some(fallback_path.clone());
                    } else {
                      // Compute file hash on demand
                      if let Ok(hash_str) = utils::calc_file_hash(book.book_path.as_pathbuf()).await
                      {
                        let hash = crate::db::models::books::BookHash(hash_str.into());
                        let hashed_path = app_dirs
                          .read()
                          .thumbnails_dir
                          .join("hashed_books")
                          .join(format!("{}.png", hash.0));
                        if hashed_path.exists() {
                          path_to_thumbnail = Some(hashed_path);
                        } else {
                          // Need to create the thumbnail — store path and hash
                          path_to_thumbnail = Some(hashed_path);
                          computed_hash = Some(hash);
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        }

        // Fallback if BookSizes is not yet initialized or found
        let path_to_thumbnail = path_to_thumbnail.unwrap_or(fallback_path);

        // Ensure parent directory exists
        if let Some(parent) = path_to_thumbnail.parent() {
          let _ = std::fs::create_dir_all(parent);
        }

        // 4. Extract PNG to disk if it doesn't exist yet
        let extraction_ok = if path_to_thumbnail.exists() {
          true
        } else {
          let res =
            mutool::extract_img::extract_img(&book.book_path.as_pathbuf(), 20, &path_to_thumbnail)
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
        };

        // 4. If extraction succeeded, update Book.has_thumbnail and send event
        if extraction_ok {
          let db_save_res = db.rw_t(|rw_t| {
            // Update Book record to have has_thumbnail = true
            if let Some(mut b) = rw_t.get().primary::<Book>(book.id.clone())? {
              let old_b = b.clone();
              b.has_thumbnail = true;
              rw_t.update::<Book>(old_b, b)?;
            }

            // If we computed a hash, migrate MutoolData → BookHash in DuplicateSize
            if let Some(hash) = &computed_hash
              && let Some(old_book_sizes) = rw_t.get().primary::<BookSizes>(book.book_size)?
            {
              let mut updated_book_sizes = old_book_sizes.clone();
              if let BookType::DuplicateSize(map) = &mut updated_book_sizes.book_type
                && let Some(MutoolData(_)) = map.get(&book.book_path)
              {
                *map.get_mut(&book.book_path).unwrap() = BookHash(hash.clone());
                rw_t.update::<BookSizes>(old_book_sizes, updated_book_sizes)?;

                // Insert or update BookHashes record
                let mut books_set = HashSet::default();
                books_set.insert(book.book_path.clone());
                let book_hashes = BookHashes::new(hash.clone(), books_set);
                if let Some(old_hashes) = rw_t.get().primary::<BookHashes>(hash.clone())? {
                  let mut updated_hashes = old_hashes.clone();
                  updated_hashes.books.insert(book.book_path.clone());
                  rw_t.update::<BookHashes>(old_hashes, updated_hashes)?;
                } else {
                  rw_t.insert::<BookHashes>(book_hashes)?;
                }
              }
            }
            Ok(())
          });

          if db_save_res.is_ok() {
            send_event!(event_tx, LibraryEvent::ThumbnailExtracted(book.book_path.clone()));
          }
        }
      }
      drop(permit);
    });
  }
}
