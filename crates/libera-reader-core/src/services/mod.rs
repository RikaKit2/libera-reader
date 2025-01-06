use crate::vars::PATH_TO_SCAN;
use std::ops::Deref;
use tracing::info;

mod dir_scan_service;
mod data_extraction_service;
pub(crate) mod notify_service;


pub struct Services {}

impl Services {
  pub(crate) fn new() -> Self { Self {} }
  pub async fn run(&mut self) {
    let start = std::time::Instant::now();
    self.launch_dir_scan_service(false).await;
    self.run_notify();
    self.run_data_extraction();
    info!("All services execution time is: {:?}", start.elapsed());
  }

  pub fn run_notify(&mut self) {
    match PATH_TO_SCAN.read().unwrap().deref() {
      None => {}
      Some(path_to_scan) => {
        notify_service::run_watcher(path_to_scan);
        tokio::spawn(notify_service::run());
      }
    }
  }
  pub fn stop_notify(&self) {
    match PATH_TO_SCAN.read().unwrap().deref() {
      None => {}
      Some(path_to_scan) => notify_service::stop_watcher(path_to_scan)
    };
  }

  pub async fn launch_dir_scan_service(&self, is_blocking: bool) {
    match PATH_TO_SCAN.read().unwrap().deref() {
      None => {}
      Some(path_to_scan) => {
        match is_blocking {
          true => {
            dir_scan_service::run(path_to_scan.clone()).await;
          }
          false => {
            tokio::spawn(dir_scan_service::run(path_to_scan.clone()));
          }
        }
      }
    }
  }
  pub fn run_data_extraction(&self) {}
}
impl Default for Services {
  fn default() -> Self { Self {} }
}
