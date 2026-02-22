use std::path::PathBuf;

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
use utils::{debug, error};

pub(crate) mod fs_handlers;

#[derive(Debug)]
enum FSEvent {
  CreateFile { file_path: PathBuf },
  RenameFile { old_path: PathBuf, new_path: PathBuf },
  RenameDir { old_path: PathBuf, new_path: PathBuf },
  RemoveFile { file_path: PathBuf },
  RemoveDir { dir_path: PathBuf },
}
impl FSEvent {
  fn from_event(event: Event) -> Option<Self> {
    // Destructure once and match on the kind directly to collapse nested matches.
    let Event { kind, paths, .. } = event;
    match kind {
      EventKind::Create(CreateKind::File) => paths.into_iter().next().map(|path| Self::CreateFile { file_path: path }),
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
      EventKind::Remove(RemoveKind::File) => paths.into_iter().next().map(|path| Self::RemoveFile { file_path: path }),
      EventKind::Remove(RemoveKind::Folder) => paths.into_iter().next().map(|path| Self::RemoveDir { dir_path: path }),
      _ => None,
    }
  }
}

pub struct NotifyService {
  status: WorkStatus,
  watcher: RecommendedWatcher,
  notify_rx: Option<tokio::sync::mpsc::UnboundedReceiver<notify::Event>>,
  not_cached_books: NotCachedBooks,
  settings: SETTINGS,
  db: DB,
}

impl NotifyService {
  pub(crate) fn new(not_cached_books: NotCachedBooks, settings: SETTINGS, db: DB) -> Result<Self> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let watcher = notify::recommended_watcher(move |res| match res {
      Ok(event) => {
        tx.send(event).unwrap();
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
          && let Some(mut rx) = self.notify_rx.take()
        {
          let db = self.db.clone();
          let not_cached_books = self.not_cached_books.clone();
          let settings = self.settings.clone();
          self.watcher.watch(path_to_scan.as_ref(), notify::RecursiveMode::Recursive)?;
          self.status = WorkStatus::Working;
          tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
              Self::event_processing(event, &settings, &not_cached_books, &db).await;
            }
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
  async fn event_processing(event: Event, settings: &SETTINGS, not_cached_books: &NotCachedBooks, db: &DB) {
    if let Some(fs_event) = FSEvent::from_event(event) {
      match fs_event {
        FSEvent::CreateFile { file_path } => {
          let start_time = std::time::Instant::now();
          match BookPath::new(&file_path) {
            Some(book_path) => match fs_handlers::insert_book(book_path, db, settings, not_cached_books).await {
              Ok(_) => {
                let total_time = start_time.elapsed();
                debug!("The total time for adding a book: {:?}", &total_time);
              }
              Err(err) => {
                debug!("Error inserting book: {:?}", err);
              }
            },
            None => {
              debug!("Error creating BookPath from file_path: {:?}", file_path);
            }
          }
        }
        FSEvent::RenameFile { old_path, new_path } => {
          if let Err(err) = fs_handlers::update_book_path(old_path, new_path, db).await {
            debug!("Error updating book path: {:?}", err);
          }
        }
        FSEvent::RenameDir { old_path, new_path } => {
          if let Err(err) = fs_handlers::update_book_dir(old_path, new_path, db) {
            debug!("Error updating book dir: {:?}", err);
          }
        }
        FSEvent::RemoveFile { file_path } => match BookPath::new(&file_path) {
          Some(book_path) => {
            if let Err(err) = fs_handlers::remove_book(book_path, db).await {
              debug!("Error removing book: {:?}", err);
            }
          }
          None => {
            debug!("Error creating BookPath from file_path: {:?}", file_path);
          }
        },
        FSEvent::RemoveDir { dir_path } => {
          if let Err(err) = fs_handlers::remove_books_in_dir(BookDir::new(dir_path), db) {
            debug!("Error removing books in dir: {:?}", err);
          }
        }
      };
    };
  }
}
