use std::collections::HashSet;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;

use crossbeam_channel::Receiver;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::RwLock;
use tracing::info;

use crate::book_manager::BookManager;
use crate::db::model::PrismaClient;

pub mod dir_scan_service;
pub mod data_extraction_service;
pub mod notify_service;


pub struct Services {
  book_manager: Arc<RwLock<BookManager>>,
  path_to_scan: Rc<RwLock<Option<String>>>,
  notify_events: Arc<Receiver<notify::Result<Event>>>,
  pub(crate) watcher: Arc<RwLock<RecommendedWatcher>>,
  target_ext: Arc<RwLock<HashSet<String>>>,
  client: Arc<PrismaClient>,
}

impl Services {
  pub fn new(path_to_scan: Rc<RwLock<Option<String>>>,
             book_manager: Arc<RwLock<BookManager>>,
             watcher: Arc<RwLock<RecommendedWatcher>>,
             notify_events: Arc<Receiver<notify::Result<Event>>>,
             target_ext: Arc<RwLock<HashSet<String>>>,
             client: Arc<PrismaClient>) -> Services {
    Services { book_manager, path_to_scan, watcher, notify_events, target_ext, client }
  }
  pub async fn run(&mut self) {
    self.run_dir_scan().await;
    self.run_notify().await;
    self.run_data_extraction().await;
  }
  pub async fn run_notify(&mut self) {
    match self.path_to_scan.read().await.deref() {
      None => {
        info!("path_to_scan is None")
      }
      Some(path_to_scan) => {
        self.watcher.write().await.watch(path_to_scan.as_ref(), RecursiveMode::Recursive).unwrap();
        tokio::spawn(notify_service::run(self.notify_events.clone(), self.book_manager.clone()));
      }
    }
  }
  pub async fn run_dir_scan(&self) {
    match self.path_to_scan.read().await.deref() {
      None => {}
      Some(path_to_scan) => {
        dir_scan_service::run(path_to_scan, self.client.clone(), &self.target_ext).await;
      }
    }
  }
  pub async fn run_data_extraction(&self) {
    match self.path_to_scan.read().await.deref() {
      None => {}
      Some(_path_to_scan) => {}
    }
  }
  pub async fn notify_unwatch(&mut self, path_to_scan: String) {
    self.watcher.write().await.unwatch(path_to_scan.as_ref()).unwrap();
  }
}
