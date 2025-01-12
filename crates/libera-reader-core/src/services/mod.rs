use crate::vars::{PATH_TO_SCAN, SHUTDOWN};
use std::ops::Deref;
use std::sync::atomic::Ordering;
use tracing::info;

mod dir_scan_service;
mod data_extraction_service;
pub(crate) mod notify_service;

pub enum ServiceStatus {
  Working,
  NotWorking,
}

pub struct Services {
  notify_working_status: ServiceStatus,
  data_extraction_service_working_status: ServiceStatus,
}

impl Services {
  pub(crate) fn new() -> Self {
    Self {
      notify_working_status: ServiceStatus::NotWorking,
      data_extraction_service_working_status: ServiceStatus::NotWorking,
    }
  }
  pub async fn run(&mut self) {
    let start_time = std::time::Instant::now();
    match PATH_TO_SCAN.read().unwrap().deref() {
      None => {}
      Some(path_to_scan) => {
        tokio::spawn(dir_scan_service::run(path_to_scan.clone()));
        match self.notify_working_status {
          ServiceStatus::NotWorking => {
            notify_service::run_watcher(path_to_scan);
            tokio::spawn(notify_service::run());
            self.notify_working_status = ServiceStatus::Working;
          }
          _ => {}
        }
        self.run_data_extraction();
      }
    }
    info!("All services execution time is: {:?}", start_time.elapsed());
  }

  pub fn run_notify(&mut self) {
    match self.notify_working_status {
      ServiceStatus::NotWorking => {
        match PATH_TO_SCAN.read().unwrap().deref() {
          None => {}
          Some(path_to_scan) => {
            notify_service::run_watcher(path_to_scan);
            tokio::spawn(notify_service::run());
          }
        }
        self.notify_working_status = ServiceStatus::Working;
      }
      _ => {}
    }
  }
  pub fn stop_notify(&mut self) {
    match self.notify_working_status {
      ServiceStatus::NotWorking => {
        match PATH_TO_SCAN.read().unwrap().deref() {
          None => {}
          Some(path_to_scan) => notify_service::stop_watcher(path_to_scan)
        };
        self.notify_working_status = ServiceStatus::Working;
      }
      _ => {}
    }
  }

  pub async fn launch_dir_scan_service(&mut self, is_blocking: bool) {
    match PATH_TO_SCAN.read().unwrap().deref() {
      None => {}
      Some(path_to_scan) => {
        match is_blocking {
          true => { dir_scan_service::run(path_to_scan.clone()).await; }
          false => { tokio::spawn(dir_scan_service::run(path_to_scan.clone())); }
        }
      }
    }
  }
  pub fn run_data_extraction(&mut self) {
    match self.data_extraction_service_working_status {
      ServiceStatus::NotWorking => {
        tokio::spawn(data_extraction_service::run());
        self.data_extraction_service_working_status = ServiceStatus::Working;
      }
      _ => {}
    }
  }
  pub fn stop_all_services(&mut self) {
    self.stop_notify();
    SHUTDOWN.swap(true, Ordering::Relaxed);
  }
}

impl Default for Services {
  fn default() -> Self { Self::new() }
}
impl Drop for Services {
  fn drop(&mut self) {
    self.stop_all_services();
  }
}
