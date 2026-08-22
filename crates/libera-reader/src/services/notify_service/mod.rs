use std::path::PathBuf;
use std::time::Duration;

use crate::{
  books_state::BooksStateHandle,
  db::{
    DB,
    models::books::{book::Book, book::BookDir, book::BookPath, book::BookSnapshot},
  },
  not_cached_books::NotCachedBooks,
  settings::SETTINGS,
};

use super::WorkStatus;
use anyhow::Result;
use notify::{
  Event, EventKind, RecommendedWatcher, Watcher,
  event::{CreateKind, ModifyKind, RemoveKind, RenameMode},
};
use tokio::sync::mpsc::UnboundedReceiver;
use utils::{debug, error};

pub(crate) mod fs_handlers;

/// Status returned when removing a book
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoveStatus {
  /// Book was fully removed from DB
  FullyDeleted,
  /// Book was marked as deleted (soft delete) but remains in DB
  MarkedAsDeleted,
}

/// Debounce interval for batching filesystem events
const DEBOUNCE_INTERVAL: Duration = Duration::from_millis(300);

#[derive(Debug, Clone)]
enum FSEvent {
  CreateFile { file_path: PathBuf },
  RenameFile { old_path: PathBuf, new_path: PathBuf },
  RenameDir { old_path: PathBuf, new_path: PathBuf },
  RemoveFile { file_path: PathBuf },
  RemoveDir { dir_path: PathBuf },
}

impl FSEvent {
  fn from_event(event: Event) -> Option<Self> {
    let Event { kind, paths, .. } = event;
    match kind {
      EventKind::Create(CreateKind::File) => {
        paths.into_iter().next().map(|path| Self::CreateFile { file_path: path })
      }
      EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
        let mut it = paths.into_iter();
        match (it.next(), it.next()) {
          (Some(old_path), Some(new_path)) => {
            if new_path.is_file() {
              Some(Self::RenameFile { old_path, new_path })
            } else if new_path.is_dir() {
              Some(Self::RenameDir { old_path, new_path })
            } else {
              None
            }
          }
          _ => None,
        }
      }
      EventKind::Remove(RemoveKind::File) => {
        paths.into_iter().next().map(|path| Self::RemoveFile { file_path: path })
      }
      EventKind::Remove(RemoveKind::Folder) => {
        paths.into_iter().next().map(|path| Self::RemoveDir { dir_path: path })
      }
      _ => None,
    }
  }
}

pub struct NotifyService {
  status: WorkStatus,
  watcher: RecommendedWatcher,
  notify_rx: Option<UnboundedReceiver<notify::Event>>,
  not_cached_books: NotCachedBooks,
  settings: SETTINGS,
  db: DB,
  app_dirs: crate::app_dirs::AppDirs,
  state_handle: Option<BooksStateHandle>,
}

impl NotifyService {
  pub(crate) fn new(
    not_cached_books: NotCachedBooks, settings: SETTINGS, db: DB,
    app_dirs: crate::app_dirs::AppDirs, state_handle: Option<BooksStateHandle>,
  ) -> Result<Self> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let watcher = notify::recommended_watcher(move |res| match res {
      Ok(event) => {
        let _ = tx.send(event);
      }
      Err(err) => {
        error!("Notify watcher error: {:?}", err)
      }
    })?;
    Ok(Self {
      status: WorkStatus::NotWorking,
      watcher,
      notify_rx: Some(rx),
      not_cached_books,
      settings,
      db,
      app_dirs,
      state_handle,
    })
  }

  pub fn set_state_handle(&mut self, handle: BooksStateHandle) {
    self.state_handle = Some(handle);
  }

  pub fn run(&mut self) -> Result<()> {
    match &self.status {
      WorkStatus::Working => {}
      WorkStatus::NotWorking => {
        if let Some(path_to_scan) = self.settings.get_path_to_scan_if_exists()
          && let Some(rx) = self.notify_rx.take()
        {
          let db = self.db.clone();
          let not_cached_books = self.not_cached_books.clone();
          let state_handle = self.state_handle.clone();
          let app_dirs = self.app_dirs.clone();

          self.watcher.watch(path_to_scan.as_ref(), notify::RecursiveMode::Recursive)?;
          self.status = WorkStatus::Working;

          tokio::spawn(async move {
            run_event_loop(rx, not_cached_books, db, app_dirs, state_handle).await;
          });
        }
      }
    };
    Ok(())
  }

  pub fn stop(&mut self) -> Result<()> {
    match &self.settings.read().previous_path_to_scan {
      None => Ok(()),
      Some(path_to_scan) => {
        self.status = WorkStatus::NotWorking;
        Ok(self.watcher.unwatch(path_to_scan.as_ref())?)
      }
    }
  }
}

/// Main event processing loop with batching using tokio::select!
async fn run_event_loop(
  mut rx: UnboundedReceiver<notify::Event>, not_cached_books: NotCachedBooks, db: DB,
  app_dirs: crate::app_dirs::AppDirs, state_handle: Option<BooksStateHandle>,
) {
  loop {
    let mut buffer: Vec<FSEvent> = Vec::new();
    let debounce_timer = tokio::time::sleep(DEBOUNCE_INTERVAL);
    tokio::pin!(debounce_timer);

    loop {
      tokio::select! {
        // Received event from channel
        maybe_event = rx.recv() => {
          if let Some(event) = maybe_event {
            if let Some(fs_event) = FSEvent::from_event(event) {
              buffer.push(fs_event);
            }
          } else {
            // Channel closed, exit outer loop
            return;
          }
        }
        // Timer expired
        _ = &mut debounce_timer => {
          break; // Exit collection loop
        }
      }
    }

    // Process buffer after timer expiration
    if !buffer.is_empty() {
      process_batch(
        buffer,
        not_cached_books.clone(),
        db.clone(),
        app_dirs.clone(),
        state_handle.clone(),
      )
      .await;
    }
  }
}

/// Process the entire batch of events in a SINGLE thread and a SINGLE DB transaction
async fn process_batch(
  batch: Vec<FSEvent>, not_cached_books: NotCachedBooks, db: DB,
  app_dirs: crate::app_dirs::AppDirs, state_handle: Option<BooksStateHandle>,
) {
  tokio::task::spawn_blocking(move || {
    let start_time = std::time::Instant::now();
    let events_count = batch.len();

    // Books that should generate events after the transaction commits. We split
    // the DB-transactional side from the event-emission side so that snapshot
    // construction can call `has_thumbnail_on_disk` (which needs an `&DB`).
    let mut added_books: Vec<Book> = Vec::new();
    let mut updated_books: Vec<Book> = Vec::new();
    let mut removed_paths: Vec<BookPath> = Vec::new();
    let mut renamed_paths: Vec<(BookPath, BookPath)> = Vec::new();
    let mut removed_dirs: Vec<BookDir> = Vec::new();

    // Open transaction ONCE for the entire batch of events
    let result = db.rw_t(|rw_t| {
      for event in batch {
        // Clean and clear match handling events in strict order of arrival
        match event {
          FSEvent::CreateFile { file_path } => {
            if let Some(book_path) = BookPath::new(&file_path)
              && let Ok(_) = fs_handlers::insert_book(book_path.clone(), rw_t, &not_cached_books)
            {
              let _ = not_cached_books.tx().send(book_path.clone());
              if let Ok(Some(book)) =
                rw_t.get().primary::<Book>(book_path.full_path_string().to_string())
              {
                added_books.push(book);
              }
            }
          }

          FSEvent::RemoveFile { file_path } => {
            // Use the new function that doesn't require reading metadata from disk
            if let Some(book_path) = BookPath::new(&file_path) {
              match fs_handlers::remove_book_by_path(&file_path, rw_t) {
                Ok(RemoveStatus::FullyDeleted) => {
                  removed_paths.push(book_path);
                }
                Ok(RemoveStatus::MarkedAsDeleted) => {
                  // Fetch updated book from DB and send BookUpdated
                  if let Ok(Some(updated_book)) =
                    rw_t.get().primary::<Book>(book_path.full_path_string().to_string())
                  {
                    updated_books.push(updated_book);
                  }
                }

                Err(err) => {
                  debug!("Error removing book {:?}: {:?}", file_path, err);
                }
              }
            }
          }

          FSEvent::RenameFile { old_path, new_path } => {
            if let Err(err) =
              fs_handlers::update_book_path(old_path.clone(), new_path.clone(), rw_t)
            {
              debug!("Error updating book path: {:?}", err);
            } else if let (Some(old_bp), Some(new_bp)) =
              (BookPath::new(&old_path), BookPath::new(&new_path))
            {
              renamed_paths.push((old_bp, new_bp));
            }
          }

          FSEvent::RenameDir { old_path, new_path } => {
            if let Err(err) = fs_handlers::update_book_dir(old_path.clone(), new_path.clone(), rw_t)
            {
              debug!("Error updating book dir: {:?}", err);
            } else {
              // Queue BookUpdated for every book in the renamed directory.
              let new_dir_path = BookDir::new(new_path.clone()).full_path().to_string();
              if let Ok(all_books) = rw_t.scan().primary::<Book>() {
                for item in all_books.all().unwrap() {
                  if let Ok(book) = item
                    && book.parent_dir == new_dir_path
                  {
                    updated_books.push(book);
                  }
                }
              }
            }
          }

          FSEvent::RemoveDir { dir_path } => {
            let dir = BookDir::new(dir_path.clone());
            if let Err(err) = fs_handlers::remove_books_in_dir(dir.clone(), rw_t) {
              debug!("Error removing books in dir: {:?}", err);
            } else {
              removed_dirs.push(dir);
            }
          }
        }
      }
      Ok(())
    });

    // Now notify UI with snapshot construction that needs `&DB`.
    let thumbnails_dir = app_dirs.read().thumbnails_dir.clone();
    if let Some(handle) = &state_handle {
      for book in added_books {
        let snapshot =
          BookSnapshot::from_book(&book, book.has_thumbnail_on_disk(&db, &thumbnails_dir));
        handle.add_book(snapshot);
      }
      for book in updated_books {
        let snapshot =
          BookSnapshot::from_book(&book, book.has_thumbnail_on_disk(&db, &thumbnails_dir));
        handle.update_book(snapshot);
      }
      for path in removed_paths {
        handle.remove_book(path);
      }
      for (old_path, new_path) in renamed_paths {
        handle.update_book_path(old_path, new_path);
      }
      for dir in removed_dirs {
        handle.remove_dir(dir);
      }
    }
    if let Err(err) = result {
      debug!("Batch processing error: {:?}", err);
    }
    debug!("Processed batch of {} events in {:?}", events_count, start_time.elapsed());
  });
}
