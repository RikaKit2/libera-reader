use std::path::PathBuf;
use std::time::Duration;

use crate::{
  db::{
    DB,
    models::books::book::{BookDir, BookPath},
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
}

impl NotifyService {
  pub(crate) fn new(not_cached_books: NotCachedBooks, settings: SETTINGS, db: DB) -> Result<Self> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let watcher = notify::recommended_watcher(move |res| match res {
      Ok(event) => {
        let _ = tx.send(event); // Игнорируем ошибку, если канал закрыт при выходе
      }
      Err(err) => {
        error!("Notify watcher error: {:?}", err)
      }
    })?;
    Ok(Self { status: WorkStatus::NotWorking, watcher, notify_rx: Some(rx), not_cached_books, settings, db })
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
          let settings = self.settings.clone();

          self.watcher.watch(path_to_scan.as_ref(), notify::RecursiveMode::Recursive)?;
          self.status = WorkStatus::Working;

          tokio::spawn(async move {
            run_event_loop(rx, settings, not_cached_books, db).await;
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

/// Main event processing loop with batching
async fn run_event_loop(
  mut rx: UnboundedReceiver<notify::Event>, settings: SETTINGS, not_cached_books: NotCachedBooks, db: DB,
) {
  let mut buffer: Vec<FSEvent> = Vec::with_capacity(256);

  loop {
    // 1. Wait indefinitely for the first event
    match rx.recv().await {
      Some(event) => {
        if let Some(fs_event) = FSEvent::from_event(event) {
          buffer.push(fs_event);
        }
      }
      None => break, // channel closed, exit
    };

    // 2. Once the first event arrives, collect "tail" events for 300ms
    loop {
      match tokio::time::timeout(DEBOUNCE_INTERVAL, rx.recv()).await {
        Ok(Some(event)) => {
          if let Some(fs_event) = FSEvent::from_event(event) {
            buffer.push(fs_event);
          }
        }
        Ok(None) => break, // channel closed
        Err(_) => break,   // timeout (300ms) elapsed, time to process the batch
      }
    }

    // 3. Process the batch
    if !buffer.is_empty() {
      // std::mem::take moves data from buffer into batch, leaving buffer empty for the next cycle
      let batch = std::mem::take(&mut buffer);
      process_batch(batch, settings.clone(), not_cached_books.clone(), db.clone()).await;
    }
  }
}

/// Process the entire batch of events in a SINGLE thread and a SINGLE DB transaction
async fn process_batch(batch: Vec<FSEvent>, settings: SETTINGS, not_cached_books: NotCachedBooks, db: DB) {
  tokio::task::spawn_blocking(move || {
    let start_time = std::time::Instant::now();
    let events_count = batch.len();

    // Open transaction ONCE for the entire batch of events
    let result = db.rw_t(|rw_t| {
      for event in batch {
        // Clean and clear match handling events in strict order of arrival
        match event {
          FSEvent::CreateFile { file_path } => {
            if let Some(book_path) = BookPath::new(&file_path)
              && let Err(err) = fs_handlers::insert_book(book_path, rw_t, &settings, &not_cached_books)
            {
              debug!("Error inserting book {:?}: {:?}", file_path, err);
            }
          }

          FSEvent::RemoveFile { file_path } => {
            // Use the new function that doesn't require reading metadata from disk
            if let Err(err) = fs_handlers::remove_book_by_path(&file_path, rw_t) {
              debug!("Error removing book {:?}: {:?}", file_path, err);
            }
          }

          FSEvent::RenameFile { old_path, new_path } => {
            if let Err(err) = fs_handlers::update_book_path(old_path, new_path, rw_t) {
              debug!("Error updating book path: {:?}", err);
            }
          }

          FSEvent::RenameDir { old_path, new_path } => {
            if let Err(err) = fs_handlers::update_book_dir(old_path, new_path, rw_t) {
              debug!("Error updating book dir: {:?}", err);
            }
          }

          FSEvent::RemoveDir { dir_path } => {
            if let Err(err) = fs_handlers::remove_books_in_dir(BookDir::new(dir_path), rw_t) {
              debug!("Error removing books in dir: {:?}", err);
            }
          }
        }
      }
      Ok(())
    });

    if let Err(err) = result {
      debug!("Batch processing error: {:?}", err);
    }
    debug!("Processed batch of {} events in {:?}", events_count, start_time.elapsed());
  })
  .await
  .ok();
}
