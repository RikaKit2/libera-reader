use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;

use notify::{Event, EventKind, RecursiveMode, Watcher};
use notify::event::{CreateKind, ModifyKind, RemoveKind, RenameMode};
use tokio::sync::RwLock;

use crate::app_state::NotCachedBook;
use crate::book_manager;
use crate::db::prisma::prisma::PrismaClient;

async fn event_processing(event: Event, target_ext: &Arc<RwLock<gxhash::HashSet<String>>>,
                          not_cached_books: &Arc<RwLock<VecDeque<NotCachedBook>>>,
                          client: &Arc<PrismaClient>) {
  match event {
    Event { kind, paths, attrs: _attrs } => {
      match kind {
        EventKind::Create(create_kind) => {
          match create_kind {
            CreateKind::File => {
              let path_to_book = paths[0].to_str().unwrap().to_string();
              let book_folder = paths[0].parent().unwrap().to_str().unwrap().to_string();
              let book_name = paths[0].file_name().unwrap().to_str().unwrap().to_string();
              let ext = paths[0].extension().unwrap().to_str().unwrap().to_string();
              book_manager::create_new_book(path_to_book, book_folder, book_name, ext,
                                            target_ext, not_cached_books, client).await;
            }
            _ => {}
          }
        }
        EventKind::Modify(modify_kind) => {
          match modify_kind {
            ModifyKind::Name(rename_mode) => {
              match rename_mode {
                RenameMode::To => {}
                RenameMode::From => {}
                RenameMode::Both => {
                  let new_path = &paths[0];
                  let old_path = &paths[1];
                  if old_path.is_dir() {
                    book_manager::rename_dir(new_path, old_path, client).await;
                  } else {
                    book_manager::rename_book_data(new_path, old_path, client).await;
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
              let path_to_book = paths[0].to_str().unwrap().to_string();
              book_manager::delete_book(path_to_book, client).await;
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

pub async fn run(target_ext: Arc<RwLock<gxhash::HashSet<String>>>,
                 not_cached_books: Arc<RwLock<VecDeque<NotCachedBook>>>,
                 client: Arc<PrismaClient>, path_to_scan: String) {
  let (tx, rx) = crossbeam_channel::bounded(20_000);
  let mut watcher = notify::recommended_watcher(move |res| {
    tx.send(res).unwrap();
  }).unwrap();
  watcher.watch(path_to_scan.as_ref(), RecursiveMode::Recursive).unwrap();
  loop {
    match rx.try_recv() {
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
    tokio::time::sleep(Duration::from_millis(100)).await;
  }
}


