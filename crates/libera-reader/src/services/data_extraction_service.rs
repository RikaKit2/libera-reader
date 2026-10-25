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
use crate::types::HashSet;
use parking_lot::Mutex;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::watch::Receiver;

const COVER_EXTRACTION_DPI: u32 = 48;

pub async fn run(
  db: DB, app_dirs: AppDirs, mut rx: UnboundedReceiver<BookPath>, books_state: BooksState,
  settings: SETTINGS, mut coordinator_rx: Receiver<ExtractionCoordinatorMode>,
) {
  let max_cpus = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
  let mut current_workers = (settings.read().workers_num.max(1) as usize).min(max_cpus);
  let semaphore = Arc::new(Semaphore::new(current_workers));
  let in_flight: Arc<Mutex<HashSet<BookPath>>> = Arc::new(Mutex::new(HashSet::default()));

  loop {
    // 1. Wait if the user is actively reading a book in BookViewer
    ExtractionCoordinator::wait_for_library_mode(&mut coordinator_rx).await;

    let Some(path) = rx.recv().await else { break };

    // Deduplicate: skip if this book is already being extracted right now
    {
      let mut active = in_flight.lock();
      if active.contains(&path) {
        continue;
      }
      active.insert(path.clone());
    }

    // Dynamic worker pool scaling based on user settings
    let target_workers = (settings.read().workers_num.max(1) as usize).min(max_cpus);
    if target_workers > current_workers {
      semaphore.add_permits(target_workers - current_workers);
      current_workers = target_workers;
    }

    let permit = match Arc::clone(&semaphore).acquire_owned().await {
      Ok(p) => p,
      Err(_) => break,
    };

    let should_forget_permit = if target_workers < current_workers {
      current_workers -= 1;
      true
    } else {
      false
    };

    // 2. Re-check coordinator state after acquiring permit
    ExtractionCoordinator::wait_for_library_mode(&mut coordinator_rx).await;

    let db = db.clone();
    let app_dirs = app_dirs.clone();
    let books_state = books_state.clone();
    let in_flight_worker = in_flight.clone();

    tokio::spawn(async move {
      if let Ok(Some(book)) = crate::db::models::books::book::Book::get(&db, path.clone()) {
        let BookSize::BYTES(book_size_bytes) = book.book_size;
        if book_size_bytes == 0 {
          in_flight_worker.lock().swap_remove(&path);
          if should_forget_permit {
            permit.forget();
          } else {
            drop(permit);
          }
          return;
        }

        if let Ok(Some(existing_path)) = book.get_thumbnail_path(&db, &app_dirs.thumbnails_dir) {
          books_state.mark_thumbnail_extracted(&book.book_path, Some(existing_path));
          in_flight_worker.lock().swap_remove(&path);
          if should_forget_permit {
            permit.forget();
          } else {
            drop(permit);
          }
          return;
        }

        let mut computed_hash = None;
        if let Ok(Some(book_sizes)) = db.get_primary::<BookSizes>(book.book_size) {
          match book_sizes.book_type {
            BookType::UniqueSize { .. } => {}
            DuplicateSize(map) => {
              if let Some(dup_data) = map.get(&book.book_path) {
                match dup_data {
                  BookHash(hash) => computed_hash = Some(hash.clone()),
                  MutoolData(_) => {
                    if let Ok(hash_str) =
                      crate::utils::calc_file_hash(book.book_path.as_pathbuf()).await
                    {
                      computed_hash = Some(crate::db::models::books::BookHash(hash_str.into()));
                    }
                  }
                }
              }
            }
          }
        }

        let path_to_thumbnail =
          app_dirs.book_cover_path(book_size_bytes, computed_hash.as_ref().map(|h| h.0.as_str()));
        if let Some(parent) = path_to_thumbnail.parent() {
          let _ = std::fs::create_dir_all(parent);
        }

        let existing_cover = app_dirs
          .find_existing_book_cover(book_size_bytes, computed_hash.as_ref().map(|h| h.0.as_str()));
        let extraction_ok = match existing_cover {
          Some(_) => true,
          None => {
            let res = mutool::extract_img::extract_img(
              &book.book_path.as_pathbuf(),
              COVER_EXTRACTION_DPI,
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
            let final_thumb = existing_cover.or(Some(path_to_thumbnail));
            books_state.mark_thumbnail_extracted(&book.book_path, final_thumb);
          }
        }
      }
      in_flight_worker.lock().swap_remove(&path);
      if should_forget_permit {
        permit.forget();
      } else {
        drop(permit);
      }
    });
  }
}
