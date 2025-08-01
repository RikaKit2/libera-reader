use crate::db::models::Book;
use crate::services::{State, Status};
use crate::settings::Settings;
use crate::types::DB;
use anyhow::Result;
use notify::event::{CreateKind, ModifyKind, RemoveKind, RenameMode};
use notify::{Event, EventHandler, EventKind, RecursiveMode, Watcher};

mod handlers;

impl State {
  pub fn run_notify(&mut self) -> Result<()> {
    match self.notify_service_working_status {
      Status::Working => {}
      Status::NotWorking => {
        match &self.settings.read().path_to_scan {
          None => {}
          Some(path_to_scan) => {
            self.watcher.watch(path_to_scan.as_ref(), RecursiveMode::Recursive)?;
            self.notify_service_working_status = Status::Working;
          }
        }
      }
    }
    Ok(())
  }
  pub fn stop_notify(&mut self) -> Result<()> {
    match &self.settings.read().old_path_to_scan {
      None => { Ok(()) }
      Some(path_to_scan) => {
        self.notify_service_working_status = Status::NotWorking;
        Ok(self.watcher.unwatch(path_to_scan.as_ref())?)
      }
    }
  }
}

pub(crate) struct NotifyEventHandler {
  settings: Settings,
  db: DB,
}
impl NotifyEventHandler {
  pub fn new(settings: Settings, db: DB) -> Self {
    Self { settings, db }
  }
  fn event_processing(&mut self, event: Event) {
    match event {
      Event { kind, paths, attrs: _attrs } => match kind {
        EventKind::Create(create_kind) => match create_kind {
          CreateKind::File => {
            self.book_adding_handler(&paths[0]).unwrap();
          }
          _ => {}
        },
        EventKind::Modify(modify_kind) => match modify_kind {
          ModifyKind::Name(rename_mode) => match rename_mode {
            RenameMode::Both => {
              let old_path = &paths[0];
              let new_path = &paths[1];
              if new_path.is_dir() {
                Book::update_books_directory(old_path, new_path, &self.db).unwrap();
              } else {
                self.book_path_update_handler(old_path, new_path).unwrap();
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
            self.book_deletion_handler(paths[0].to_str().unwrap()).unwrap();
          }
          RemoveKind::Folder => {
            self.dir_deletion_handler(paths[0].to_str().unwrap().to_string()).unwrap();
          }
          _ => {}
        },
        _ => {}
      },
    }
  }
}
impl EventHandler for NotifyEventHandler {
  fn handle_event(&mut self, event: notify::Result<Event>) {
    self.event_processing(event.unwrap())
  }
}

