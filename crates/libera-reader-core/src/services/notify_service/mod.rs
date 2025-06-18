use crate::app_dirs::AppDirs;
use crate::db::crud;
use crate::db::models::TargetExt;
use crate::types::{APP_DIRS, Error, NotifyEvents, TARGET_EXT, DB};
use crate::vars;
use crossbeam_channel::Receiver;
use native_db::Database;
use notify::event::{CreateKind, ModifyKind, RemoveKind, RenameMode};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::atomic::Ordering;
use std::sync::{Arc, RwLock};
use std::thread;

mod handlers;

pub struct NotifyService {
  notify_events: Receiver<NotifyEvents>,
  path_to_scan: Option<String>,
  pub watcher: RecommendedWatcher,
  handle: Option<thread::JoinHandle<()>>,
  target_ext: TARGET_EXT,
  app_dirs: APP_DIRS,
}
impl NotifyService {
  pub fn new(target_ext: TARGET_EXT, app_dirs: APP_DIRS) -> Self {
    let (tx, rx) = crossbeam_channel::unbounded();
    let watcher = notify::recommended_watcher(move |res| tx.send(res).unwrap()).unwrap();
    Self { notify_events: rx, path_to_scan: None, watcher, handle: None, target_ext, app_dirs }
  }
  pub fn run(&mut self, db: DB, path_to_scan: String) -> Result<(), Error> {
    self.path_to_scan = Some(path_to_scan.clone());
    self.watcher.watch(path_to_scan.as_ref(), RecursiveMode::Recursive).unwrap();
    let notify_events = self.notify_events.clone();
    let target_ext = self.target_ext.clone();
    let app_dirs = self.app_dirs.clone();
    self.handle = Some(thread::spawn(move || {
      loop {
        match vars::SHUTDOWN.load(Ordering::Relaxed) {
          true => { break; }
          false => {
            match notify_events.recv() {
              Ok(res) => {
                match res {
                  Ok(event) => {
                    Self::event_processing(event, &db, &target_ext, &app_dirs);
                  }
                  Err(_) => {}
                }
              }
              Err(_) => {}
            }
          }
        };
      }
    }));
    Ok(())
  }
  fn event_processing(event: Event, db: &Database, target_ext: &Arc<RwLock<TargetExt>>, app_dirs: &Arc<RwLock<AppDirs>>) {
    match event {
      Event { kind, paths, attrs: _attrs } => {
        match kind {
          EventKind::Create(create_kind) => {
            match create_kind {
              CreateKind::File => { handlers::book_adding_handler(&paths[0], db, target_ext); }
              _ => {}
            }
          }
          EventKind::Modify(modify_kind) => {
            match modify_kind {
              ModifyKind::Name(rename_mode) => {
                match rename_mode {
                  RenameMode::Both => {
                    let old_path = &paths[0];
                    let new_path = &paths[1];
                    if new_path.is_dir() {
                      crud::book::update_the_books_directory(old_path, new_path, db);
                    } else {
                      handlers::book_path_update_handler(old_path, new_path, db, target_ext);
                    }
                  }
                  RenameMode::From => {}
                  RenameMode::To => {}
                  _ => {}
                }
              }
              _ => {}
            }
          }
          EventKind::Remove(remove_kind) => {
            match remove_kind {
              RemoveKind::File => { handlers::book_deletion_handler(paths[0].to_str().unwrap(), app_dirs, db); }
              RemoveKind::Folder => { handlers::dir_deletion_handler(paths[0].to_str().unwrap().to_string(), app_dirs, db); }
              _ => {}
            }
          }
          _ => {}
        }
      }
    }
  }
}
impl Drop for NotifyService {
  fn drop(&mut self) {
    vars::SHUTDOWN.store(true, Ordering::Relaxed);
    if let Some(handle) = self.handle.take() {
      let _ = handle.join();
    }
  }
}
