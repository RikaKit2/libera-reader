use crate::db::DB;
use crate::db::models::Book;
use crate::services::{Services, Status};
use crate::settings::Settings;
use crate::types::NotCachedBooks;
use anyhow::Result;
use notify::EventKind;
use notify::event::{CreateKind, ModifyKind, RemoveKind, RenameMode};
use notify::{Event, RecursiveMode, Watcher};
use tokio::sync::mpsc::UnboundedReceiver;

mod handlers;

impl Services {
  pub fn run_notify(&mut self) -> Result<()> {
    match self.notify_service_working_status {
      Status::Working => {}
      Status::NotWorking => match &self.settings.read().path_to_scan {
        None => {}
        Some(path_to_scan) => {
          self.watcher.watch(path_to_scan.as_ref(), RecursiveMode::Recursive)?;
          self.notify_service_working_status = Status::Working;
        }
      },
    }
    Ok(())
  }
  pub fn stop_notify(&mut self) -> Result<()> {
    match &self.settings.read().old_path_to_scan {
      None => Ok(()),
      Some(path_to_scan) => {
        self.notify_service_working_status = Status::NotWorking;
        Ok(self.watcher.unwatch(path_to_scan.as_ref())?)
      }
    }
  }
}

pub(crate) async fn run(settings: Settings, not_cached_books: NotCachedBooks, db: DB, mut rx: UnboundedReceiver<notify::Event>) {
  tokio::spawn(async move {
    loop {
      match rx.try_recv() {
        Ok(event) => {
          event_processing(event, &settings, &not_cached_books, &db).await;
        }
        Err(_) => {}
      }
    }
  });
}

async fn event_processing(event: Event, settings: &Settings, not_cached_books: &NotCachedBooks, db: &DB) {
  match event {
    Event { kind, paths, attrs: _attrs } => match kind {
      EventKind::Create(create_kind) => match create_kind {
        CreateKind::File => {
          handlers::book_adding_handler(&paths[0], settings, not_cached_books, db).await.unwrap();
        }
        _ => {}
      },
      EventKind::Modify(modify_kind) => match modify_kind {
        ModifyKind::Name(rename_mode) => match rename_mode {
          RenameMode::Both => {
            let old_path = &paths[0];
            let new_path = &paths[1];
            if new_path.is_dir() {
              Book::update_books_directory(old_path, new_path, db).unwrap();
            } else {
              handlers::book_path_update_handler(old_path, new_path, settings, not_cached_books, db).await.unwrap();
            }
          }
          RenameMode::From => {}
          RenameMode::To => {}
          _ => {}
        },
        _ => {}
      },
      EventKind::Remove(remove_kind) => match remove_kind {
        RemoveKind::File => {
          handlers::book_deletion_handler(paths[0].to_str().unwrap(), db).unwrap();
        }
        RemoveKind::Folder => {
          handlers::dir_deletion_handler(paths[0].to_str().unwrap().to_string(), db).unwrap();
        }
        _ => {}
      },
      _ => {}
    },
  }
}
