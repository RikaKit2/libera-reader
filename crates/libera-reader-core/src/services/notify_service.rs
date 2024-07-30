use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::Receiver;
use notify::{Event, EventKind};
use notify::event::{CreateKind, ModifyKind, RemoveKind, RenameMode};
use tokio::sync::RwLock;

use crate::book_manager::BookManager;
use crate::utils::BookData;

async fn event_processing(event: Event, book_manager: &Arc<RwLock<BookManager>>) {
  tracing::debug!("==========");
  tracing::debug!("{:?}", &event.kind);
  match event {
    Event { kind, paths, attrs: _attrs } => {
      match kind {
        EventKind::Create(create_kind) => {
          match create_kind {
            CreateKind::File => {
              let new_path = &paths[0];
              let new_data = BookData::from_pathbuf(new_path);
              book_manager.write().await.add_book(new_data).await;
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
                    book_manager.write().await.rename_dir(old_dir_path, new_path).await;
                  } else {
                    let old_data = BookData::from_pathbuf(old_path);
                    let new_data = BookData::from_pathbuf(new_path);
                    book_manager.write().await.rename_data(old_data, new_data).await;
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
          tracing::debug!("{:?}", &event.kind);
          match remove_kind {
            RemoveKind::File => {
              let old_path_to_book = paths[0].to_str().unwrap().to_string();
              book_manager.write().await.delete_book(old_path_to_book).await;
            }
            RemoveKind::Folder => {
              book_manager.write().await.delete_dir(paths).await;
            }
            _ => {}
          }
        }
        _ => {}
      }
    }
  }
}

pub async fn run(notify_events: Arc<Receiver<notify::Result<Event>>>, book_manager: Arc<RwLock<BookManager>>) {
  loop {
    match notify_events.try_recv() {
      Ok(res) => {
        match res {
          Ok(event) => {
            event_processing(event, &book_manager).await;
          }
          Err(_) => {}
        }
      }
      Err(_) => {}
    }
    tokio::time::sleep(Duration::from_millis(1)).await;
  }
}

