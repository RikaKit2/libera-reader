use crate::settings::TargetExtensions;
use crate::utils::NotCachedBook;
use crossbeam_channel::Receiver;
use notify::Event;
use once_cell::sync::Lazy;
use std::collections::VecDeque;
use std::ops::Deref;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{event, Level};

mod dir_scan_service;
mod data_extraction_service;
mod notify_service;

pub(crate) static NOT_CACHED_BOOKS: Lazy<Arc<RwLock<VecDeque<NotCachedBook>>>> = Lazy::new(|| Default::default());


pub struct Services {
  path_to_scan: Arc<RwLock<Option<String>>>,
  notify_receiver: Receiver<notify::Result<Event>>,
  target_ext: TargetExtensions,
}

impl Services {
  pub fn new(path_to_scan: Arc<RwLock<Option<String>>>,
             target_ext: TargetExtensions,
             notify_receiver: Receiver<notify::Result<Event>>) -> Services {
    Services { path_to_scan, target_ext, notify_receiver }
  }
  pub async fn run(&mut self) {
    let start = std::time::Instant::now();
    self.run_dir_scan().await;
    self.run_notify().await;
    self.run_data_extraction().await;
    event!(Level::INFO, "All services execution time is: {:?}", start.elapsed());
  }

  pub async fn run_notify(&mut self) {
    match self.path_to_scan.read().await.is_some() {
      true => {
        tokio::spawn(notify_service::run(self.notify_receiver.clone(), self.target_ext.clone()));
      }
      false => {}
    }
  }

  pub async fn run_dir_scan(&self) {
    match self.path_to_scan.read().await.deref() {
      None => {}
      Some(path_to_scan) => {
        // dir_scan_service::run(path_to_scan.clone()).await;
        tokio::spawn(dir_scan_service::run(path_to_scan.clone()));
      }
    }
  }
  pub async fn run_data_extraction(&self) {}
}
