use crate::app_dirs::AppDirs;
use crate::db::DB;
use crate::db::models::books::BookType::{self, DuplicateSize};
use crate::db::models::books::DuplicateBookData::{BookHash, MutoolData};
use crate::db::models::books::book::{Book, BookPath, BookSize};
use crate::db::models::books::book_hashes::BookHashes;
use crate::db::models::books::book_sizes::BookSizes;
use crate::db::models::books::thumbnail::Thumbnail;
use crate::settings::SETTINGS;
use crate::types::{HashSet, LibraryEvent};
use std::fs::remove_file;
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

        // 2. Check if thumbnail is already in DB before extracting!
        if let Ok(Some(_)) = book.get_thumbnail_data(&db) {
          // Already extracted, just send the event to let UI know and skip extraction
          let _ = event_tx.send(LibraryEvent::ThumbnailExtracted(book.book_path.clone()));
          drop(permit);
          return;
        }

        // Determine correct thumbnail path based on book_sizes and book_hashes (hashing on demand if needed)
        let mut path_to_thumbnail = None;
        let mut computed_hash = None;

        if let Ok(Some(book_sizes)) = db.get_primary::<BookSizes>(book.book_size) {
          match book_sizes.book_type {
            BookType::UniqueSize { .. } => {
              let BookSize::BYTES(book_size_bytes) = book.book_size;
              path_to_thumbnail = Some(
                app_dirs
                  .read()
                  .thumbnails_dir
                  .join("unhashed_books")
                  .join(format!("{}.png", book_size_bytes)),
              );
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
                    // This is duplicate size but has NO hash computed yet!
                    // Compute file hash on demand (in background thread)
                    if let Ok(hash_str) = utils::calc_file_hash(book.book_path.as_pathbuf()).await {
                      let hash = crate::db::models::books::BookHash(hash_str.into());
                      path_to_thumbnail = Some(
                        app_dirs
                          .read()
                          .thumbnails_dir
                          .join("hashed_books")
                          .join(format!("{}.png", hash.0)),
                      );
                      computed_hash = Some(hash);
                    }
                  }
                }
              }
            }
          }
        }

        // Fallback if BookSizes is not yet initialized or found
        let path_to_thumbnail = path_to_thumbnail.unwrap_or_else(|| {
          let BookSize::BYTES(book_size_bytes) = book.book_size;
          app_dirs
            .read()
            .thumbnails_dir
            .join("unhashed_books")
            .join(format!("{}.png", book_size_bytes))
        });

        // Ensure parent directory exists
        if let Some(parent) = path_to_thumbnail.parent() {
          let _ = std::fs::create_dir_all(parent);
        }

        // 2. Fetch thumbnail bytes (checking if the hashed/unhashed .png already exists on disk first)
        let bytes_opt = if path_to_thumbnail.exists() {
          // If PNG already exists, read it and convert to JPEG bytes without mutool
          match mutool::extract_img::imp_to_jpeg_public(&path_to_thumbnail) {
            Ok(bytes) => Some(bytes),
            Err(_) => {
              // Failsafe: if reading/converting existing PNG fails, delete it so it can be re-extracted
              let _ = remove_file(&path_to_thumbnail);
              None
            }
          }
        } else {
          // Extract PNG using mutool to path_to_thumbnail (and keep it on disk for testing)
          let res =
            mutool::extract_img::extract_img(&book.book_path.as_pathbuf(), 20, &path_to_thumbnail)
              .await;

          if res.is_ok() {
            mutool::extract_img::imp_to_jpeg_public(&path_to_thumbnail).ok()
          } else {
            // Save the mutool error to BookSizes/BookHashes so we know extraction failed
            let err = res.err();
            let _ = db.rw_t(|rw_t| {
              if let Some(old_book_sizes) = rw_t.get().primary::<BookSizes>(book.book_size)? {
                let mut updated_book_sizes = old_book_sizes.clone();
                match &mut updated_book_sizes.book_type {
                  BookType::UniqueSize { mutool_data, .. } => {
                    let mut md = mutool_data.take().unwrap_or_default();
                    md.mutool_err = err.clone();
                    *mutool_data = Some(md);
                    rw_t.update::<BookSizes>(old_book_sizes, updated_book_sizes)?;
                  }
                  DuplicateSize(map) => {
                    if let Some(dup_data) = map.get_mut(&book.book_path) {
                      match dup_data {
                        MutoolData(m) => {
                          let mut md = m.take().unwrap_or_default();
                          md.mutool_err = err.clone();
                          *m = Some(md);
                          rw_t.update::<BookSizes>(old_book_sizes, updated_book_sizes)?;
                        }
                        BookHash(hash) => {
                          if let Some(old_book_hashes) =
                            rw_t.get().primary::<BookHashes>(hash.clone())?
                          {
                            let mut updated_book_hashes = old_book_hashes.clone();
                            updated_book_hashes.mutool_data.mutool_err = err.clone();
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
            None
          }
        };

        if let Some(bytes) = bytes_opt {
          let thumbnail = Thumbnail { data: bytes };

          // 3. Save thumbnail directly into DB models (BookSizes or BookHashes) and update Book
          let db_save_res = db.rw_t(|rw_t| {
            // Update Book record to have has_thumbnail = true
            if let Some(mut b) = rw_t.get().primary::<Book>(book.id.clone())? {
              let old_b = b.clone();
              b.has_thumbnail = true;
              rw_t.update::<Book>(old_b, b)?;
            }

            if let Some(old_book_sizes) = rw_t.get().primary::<BookSizes>(book.book_size)? {
              let mut updated_book_sizes = old_book_sizes.clone();
              match &mut updated_book_sizes.book_type {
                BookType::UniqueSize { mutool_data, .. } => {
                  let mut md = mutool_data.take().unwrap_or_default();
                  md.thumbnail = Some(thumbnail.clone());
                  *mutool_data = Some(md);
                  rw_t.update::<BookSizes>(old_book_sizes, updated_book_sizes)?;
                }
                DuplicateSize(map) => {
                  if let Some(dup_data) = map.get_mut(&book.book_path) {
                    match dup_data {
                      MutoolData(m) => {
                        // If we computed a hash on demand, migrate the DuplicateBookData to BookHash(hash)
                        if let Some(hash) = &computed_hash {
                          *dup_data = BookHash(hash.clone());
                          rw_t.update::<BookSizes>(old_book_sizes, updated_book_sizes)?;

                          // Now insert or update the BookHashes record with the thumbnail
                          let mut books_set = HashSet::default();
                          books_set.insert(book.book_path.clone());
                          let mut book_hashes = BookHashes::new(hash.clone(), books_set);
                          book_hashes.mutool_data.thumbnail = Some(thumbnail.clone());

                          if let Some(old_hashes) =
                            rw_t.get().primary::<BookHashes>(hash.clone())?
                          {
                            let mut updated_hashes = old_hashes.clone();
                            updated_hashes.books.insert(book.book_path.clone());
                            updated_hashes.mutool_data.thumbnail = Some(thumbnail.clone());
                            rw_t.update::<BookHashes>(old_hashes, updated_hashes)?;
                          } else {
                            rw_t.insert::<BookHashes>(book_hashes)?;
                          }
                        } else {
                          // Fallback if no hash computed
                          let mut md = m.take().unwrap_or_default();
                          md.thumbnail = Some(thumbnail.clone());
                          *m = Some(md);
                          rw_t.update::<BookSizes>(old_book_sizes, updated_book_sizes)?;
                        }
                      }
                      BookHash(hash) => {
                        if let Some(old_book_hashes) =
                          rw_t.get().primary::<BookHashes>(hash.clone())?
                        {
                          let mut updated_book_hashes = old_book_hashes.clone();
                          updated_book_hashes.mutool_data.thumbnail = Some(thumbnail.clone());
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

          if db_save_res.is_ok() {
            let _ = event_tx.send(LibraryEvent::ThumbnailExtracted(book.book_path.clone()));
          }
        }
      }
      drop(permit);
    });
  }
}
