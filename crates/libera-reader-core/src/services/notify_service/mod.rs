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
use tracing::{error, info};

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
    match event {
      Event { kind, paths, attrs: _attrs } => match kind {
        EventKind::Create(create_kind) => match create_kind {
          CreateKind::File => match paths.into_iter().next() {
            Some(path) => Some(Self::CreateFile { file_path: path }),
            None => None,
          },
          _ => None,
        },
        EventKind::Modify(modify_kind) => match modify_kind {
          ModifyKind::Name(rename_mode) => match rename_mode {
            RenameMode::Both => {
              let mut it = paths.into_iter();
              let poss_old_path = it.next();
              let poss_new_path = it.next();
              match (poss_old_path, poss_new_path) {
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
            RenameMode::From => None,
            RenameMode::To => None,
            _ => None,
          },
          _ => None,
        },
        EventKind::Remove(remove_kind) => match remove_kind {
          RemoveKind::File => match paths.into_iter().next() {
            Some(path) => Some(Self::RemoveFile { file_path: path }),
            None => None,
          },
          RemoveKind::Folder => match paths.into_iter().next() {
            Some(path) => Some(Self::RemoveDir { dir_path: path }),
            None => None,
          },
          _ => None,
        },
        _ => None,
      },
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
  pub fn run(&mut self, path_to_scan: &String) -> Result<()> {
    match &self.status {
      WorkStatus::Working => {}
      WorkStatus::NotWorking => match self.notify_rx.take() {
        Some(mut rx) => {
          let db = self.db.clone();
          let not_cached_books = self.not_cached_books.clone();
          let settings = self.settings.clone();
          self.watcher.watch(path_to_scan.as_ref(), notify::RecursiveMode::Recursive)?;
          // mark service as working so subsequent run() calls are no-ops
          self.status = WorkStatus::Working;
          tokio::spawn(async move {
            // Await incoming events instead of busy-looping on try_recv.
            while let Some(event) = rx.recv().await {
              Self::event_processing(event, &settings, &not_cached_books, &db).await;
            }
          });
        }
        None => {}
      },
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
    match FSEvent::from_event(event) {
      Some(fs_event) => {
        match fs_event {
          FSEvent::CreateFile { file_path } => {
            let start_time = std::time::Instant::now();
            fs_handlers::insert_book(BookPath::new(file_path), db, settings, not_cached_books).await.unwrap();
            let total_time = start_time.elapsed();
            info!("The total time for adding a book: {:?}", &total_time);
          }
          FSEvent::RenameFile { old_path, new_path } => {
            fs_handlers::update_book_path(old_path, new_path, db).await.unwrap();
          }
          FSEvent::RenameDir { old_path, new_path } => {
            fs_handlers::update_book_dir(old_path, new_path, db).unwrap();
          }
          FSEvent::RemoveFile { file_path } => {
            fs_handlers::remove_book(BookPath::new(file_path), db).await.unwrap();
          }
          FSEvent::RemoveDir { dir_path } => {
            fs_handlers::remove_books_in_dir(BookDir::new(dir_path), db).unwrap();
          }
        };
      }
      None => {}
    };
  }
}
