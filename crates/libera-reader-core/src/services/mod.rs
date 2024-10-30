use crate::app_core::{BookHashSet, BookSizeSet};
use crossbeam_channel::Receiver;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::ops::Deref;
use std::sync::Arc;
use tokio::sync::RwLock;

mod dir_scan_service;
mod data_extraction_service;
mod notify_service;


pub struct Services {
  path_to_scan: Arc<RwLock<Option<String>>>,
  watcher: RecommendedWatcher,
  notify_receiver: Arc<Receiver<notify::Result<Event>>>,
  book_sizes: BookSizeSet,
  book_hashes: BookHashSet,
}

impl Services {
  pub fn new(path_to_scan: Arc<RwLock<Option<String>>>,
             book_sizes: BookSizeSet, book_hashes: BookHashSet) -> Services {
    let (watcher, notify_receiver) =
      notify_service::get_watcher_and_receiver();
    Services {
      path_to_scan,
      watcher,
      notify_receiver,
      book_sizes,
      book_hashes,
    }
  }
  pub async fn run(&mut self) {
    self.run_dir_scan().await;
    self.run_notify().await;
    self.run_data_extraction().await;
  }

  pub async fn run_notify(&mut self) {
    match self.path_to_scan.read().await.deref() {
      None => {}
      Some(path_to_scan) => {
        self.watcher.watch(path_to_scan.as_ref(), RecursiveMode::Recursive).unwrap();
        // tokio::spawn(notify_service::run(self.notify_receiver.clone()));
      }
    }
  }

  pub async fn stop_notify(&mut self, path_to_scan: &String) {
    self.watcher.unwatch(path_to_scan.as_ref()).unwrap();
  }
  pub async fn run_dir_scan(&self) {
    // dir_scan_service::run(self.db, self.path_to_scan.into(), &self.db_book_data, &self
    //   .hashed_book_data);
  }
  pub async fn run_data_extraction(&self) {}
}
