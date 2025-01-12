use crate::vars::WATCHER;
use crate::vars::{NOTIFY_EVENTS, SHUTDOWN};
use notify::event::{CreateKind, ModifyKind, RemoveKind, RenameMode};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::sync::atomic::Ordering;
use tracing::debug;

pub(crate) mod handlers;


fn event_processing(event: Event) {
  match event {
    Event { kind, paths, attrs: _attrs } => {
      match kind {
        EventKind::Create(create_kind) => {
          match create_kind {
            CreateKind::File => {
              handlers::book_adding_handler(&paths[0]);
            }
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
                    let old_dir_path = old_path.to_str().unwrap().to_string();
                    handlers::dir_renaming_handler(old_dir_path, new_path);
                  } else {
                    handlers::book_path_update_handler(old_path, new_path);
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
            RemoveKind::File => {
              handlers::book_deletion_handler(paths[0].to_str().unwrap());
            }
            RemoveKind::Folder => {
              handlers::dir_deletion_handler(paths[0].to_str().unwrap().to_string());
            }
            _ => {}
          }
        }
        _ => {}
      }
    }
  }
}

pub async fn run() {
  loop {
    match NOTIFY_EVENTS.pop() {
      Ok(res) => {
        match res {
          Ok(event) => {
            event_processing(event);
          }
          Err(_) => {}
        }
      }
      Err(_) => {}
    }
    if SHUTDOWN.load(Ordering::Relaxed) == true {
      debug!("notify has been stopped");
      break;
    } else { continue; }
  }
}

pub fn run_watcher(path_to_scan: &String) {
  WATCHER.lock().unwrap().watch(path_to_scan.as_ref(), RecursiveMode::Recursive).unwrap();
}
pub fn stop_watcher(path_to_scan: &String) {
  match WATCHER.lock().unwrap().unwatch(path_to_scan.as_ref()) {
    Ok(_) => {}
    Err(_) => {}
  }
}
