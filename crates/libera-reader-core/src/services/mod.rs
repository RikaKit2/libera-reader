use crate::services::{Status::NotWorking, Status::Working};
use crate::vars::{SETTINGS, SHUTDOWN};
use std::sync::atomic::Ordering;
use std::thread;
use tracing::debug;

mod dir_scan_service;
pub(crate) mod notify_service;
mod data_extraction_service;

pub enum Status {
  Working,
  NotWorking,
}

pub struct Services {
  notify_working_status: Status,
  data_extraction_service_working_status: Status,
}

impl Services {
  pub fn new() -> Self {
    Self {
      notify_working_status: NotWorking,
      data_extraction_service_working_status: NotWorking,
    }
  }
  pub fn run(&mut self) {
    match &SETTINGS.read().unwrap().path_to_scan {
      None => {}
      Some(path_to_scan) => {
        match self.notify_working_status {
          NotWorking => {
            notify_service::run_watcher(&path_to_scan).unwrap();
            thread::spawn(|| { notify_service::run() });
            self.notify_working_status = Working;
          }
          _ => {}
        }
        thread::spawn(|| { dir_scan_service::run() });
        self.run_data_extraction();
      }
    }
  }

  pub fn run_notify(&mut self) {
    match self.notify_working_status {
      NotWorking => {
        match &SETTINGS.read().unwrap().path_to_scan {
          None => {}
          Some(path_to_scan) => {
            notify_service::run_watcher(path_to_scan).unwrap();
            thread::spawn(|| { notify_service::run() });
          }
        }
        self.notify_working_status = Working;
      }
      _ => {}
    }
  }
  pub fn stop_notify(&mut self) {
    match self.notify_working_status {
      NotWorking => {
        match &SETTINGS.read().unwrap().path_to_scan {
          None => {}
          Some(path_to_scan) => notify_service::stop_watcher(path_to_scan)
        };
        self.notify_working_status = Working;
      }
      _ => {}
    }
  }

  pub fn launch_dir_scan_service(&mut self, is_blocking: bool) {
    match &SETTINGS.read().unwrap().path_to_scan {
      None => { debug!("Path to scan is none") }
      Some(_path_to_scan) => {
        match is_blocking {
          true => { dir_scan_service::run(); }
          false => { thread::spawn(|| { dir_scan_service::run() }); }
        }
      }
    }
  }
  pub fn run_data_extraction(&mut self) {
    match self.data_extraction_service_working_status {
      NotWorking => {
        self.data_extraction_service_working_status = Working;
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
