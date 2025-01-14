use crate::vars::{PATH_TO_SCAN, SHUTDOWN};
use std::ops::Deref;
use std::sync::atomic::Ordering;
use std::thread;


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
  pub fn run(&mut self) {
    match PATH_TO_SCAN.read().unwrap().deref().clone() {
      None => {}
      Some(path_to_scan) => {
        match self.notify_working_status {
          ServiceStatus::NotWorking => {
            notify_service::run_watcher(&path_to_scan);
            thread::spawn(|| { notify_service::run() });
            self.notify_working_status = ServiceStatus::Working;
          }
          _ => {}
        }
        thread::spawn(|| { dir_scan_service::run(path_to_scan) });
        self.run_data_extraction();
      }
    }
  }

  pub fn run_notify(&mut self) {
    match self.notify_working_status {
      ServiceStatus::NotWorking => {
        match PATH_TO_SCAN.read().unwrap().deref() {
          None => {}
          Some(path_to_scan) => {
            notify_service::run_watcher(path_to_scan);
            thread::spawn(|| { notify_service::run() });
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

  pub fn launch_dir_scan_service(&mut self, is_blocking: bool) {
    match PATH_TO_SCAN.read().unwrap().deref().clone() {
      None => {}
      Some(path_to_scan) => {
        match is_blocking {
          true => { dir_scan_service::run(path_to_scan); }
          false => { thread::spawn(|| { dir_scan_service::run(path_to_scan) }); }
        }
      }
    }
  }
  pub fn run_data_extraction(&mut self) {
    match self.data_extraction_service_working_status {
      ServiceStatus::NotWorking => {
        thread::spawn(|| { data_extraction_service::run() });
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
