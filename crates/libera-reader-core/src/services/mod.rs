use crate::services::notify_service::watchdog;
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
    self.run_dir_scan();
    self.run_notify();
    self.run_data_extraction();
    info!("All services execution time is: {:?}", start.elapsed());
  }

  pub fn run_notify(&mut self) {
    match PATH_TO_SCAN.read().unwrap().deref() {
      None => {}
      Some(path_to_scan) => {
        watchdog::run(path_to_scan);
        tokio::spawn(notify_service::run());
      }
    }
  }
  pub fn stop_notify(&self) {
    match PATH_TO_SCAN.read().unwrap().deref() {
      None => {}
      Some(path_to_scan) => {
        match watchdog::stop(path_to_scan) {
          Ok(_) => {}
          Err(_) => {}
        }
      }
    };
  }

  pub fn run_dir_scan(&self) {
    match PATH_TO_SCAN.read().unwrap().deref() {
      None => {}
      Some(path_to_scan) => {
        tokio::spawn(dir_scan_service::run(path_to_scan.clone()));
      }
    }
  }
  pub fn run_data_extraction(&self) {}
}
impl Default for Services {
  fn default() -> Self { Self {} }
}
