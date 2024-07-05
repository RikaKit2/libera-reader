use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::Receiver;
use notify::{Event, EventKind};
use notify::event::{CreateKind, ModifyKind, RemoveKind, RenameMode};
use tokio::sync::RwLock;

use crate::book_manager;
use crate::db::model::PrismaClient;
use crate::utils::NotCachedBook;

async fn event_processing(event: Event, target_ext: &Arc<RwLock<HashSet<String>>>,
                          not_cached_books: &Arc<RwLock<VecDeque<NotCachedBook>>>,
                          client: &PrismaClient) {
  match event {
    Event { kind, paths, attrs: _attrs } => {
      match kind {
        EventKind::Create(create_kind) => {
          match create_kind {
            CreateKind::File => {
              book_manager::create_new_book(&paths[0], target_ext, not_cached_books, client).await;
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
                    book_manager::rename_dir(old_dir_path, new_path, client).await;
                  } else {
                    book_manager::rename_book_data(old_path, new_path, client, target_ext, not_cached_books).await;
                  }
                }
                _ => {}
              }
            }
            _ => {}
          }
        }
        EventKind::Remove(remove_kind) => {
          match remove_kind {
            RemoveKind::File => {
              let old_path_to_book = paths[0].to_str().unwrap().to_string();
              book_manager::delete_book(old_path_to_book, client).await;
            }
            RemoveKind::Folder => {
              book_manager::delete_dir(paths, client).await;
            }
            _ => {}
          }
        }
        _ => {}
      }
    }
  }
}


pub async fn run(target_ext: Arc<RwLock<HashSet<String>>>,
                 not_cached_books: Arc<RwLock<VecDeque<NotCachedBook>>>,
                 client: Arc<PrismaClient>,
                 notify_events: Arc<RwLock<Receiver<notify::Result<Event>>>>) {
  loop {
    match notify_events.write().await.try_recv() {
      Ok(res) => {
        match res {
          Ok(event) => {
            event_processing(event, &target_ext, &not_cached_books, &client).await;
          }
          Err(_) => {}
        }
      }
      Err(_) => {}
    }
    tokio::time::sleep(Duration::from_millis(1)).await;
  }
}


