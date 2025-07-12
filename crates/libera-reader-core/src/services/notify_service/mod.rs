use crate::db::crud;
use crate::services::Services;
use crate::types::{NotCachedBooks, APP_DIRS, DB, TARGET_EXT};
use anyhow::Result;
use notify::event::{CreateKind, ModifyKind, RemoveKind, RenameMode};
use notify::{Event, EventHandler, EventKind, RecursiveMode, Watcher};

mod handlers;

impl Services {
  pub fn run_notify(&mut self, path_to_scan: String) -> Result<()> {
    self.watcher.watch(path_to_scan.as_ref(), RecursiveMode::Recursive)?;
    Ok(())
  }
  pub fn stop_notify(&mut self) -> Result<()> {
    match &self.path_to_scan {
      None => { Ok(()) }
      Some(path_to_scan) => { Ok(self.watcher.unwatch(path_to_scan.as_ref())?) }
    }
  }
}

pub(crate) struct NotifyEventHandler {
  not_cached_books: NotCachedBooks,
  target_ext: TARGET_EXT,
  app_dirs: APP_DIRS,
  db: DB,
}
impl NotifyEventHandler {
  pub fn new(not_cached_books: NotCachedBooks, target_ext: TARGET_EXT, app_dirs: APP_DIRS, db: DB) -> Self {
    Self {
      not_cached_books,
      target_ext,
      app_dirs,
      db,
    }
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
                crud::book::update_the_books_directory(old_path, new_path, &self.db).unwrap();
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

