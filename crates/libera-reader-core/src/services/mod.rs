use crate::services::data_extraction_service::DataExtractionService;
use crate::services::notify_service::NotifyService;
use crate::types::{APP_DIRS, Error, NotCachedBooks, TARGET_EXT, DB};
use crate::vars::SHUTDOWN;
use std::sync::atomic::Ordering;

mod notify_service;
mod data_extraction_service;
mod dir_scan_service;

pub enum Status {
  Working,
  NotWorking,
}

pub(crate) enum RayonTaskType {
  ImgExtract,
  HashCalc,
}
pub(crate) fn get_num_of_threads(rayon_task_type: RayonTaskType) -> usize {
  let num_of_cpus = num_cpus::get();
  let num_threads_for_task: usize;
  if num_of_cpus >= 6 {
    match rayon_task_type {
      RayonTaskType::ImgExtract => { num_threads_for_task = num_of_cpus - 2; }
      RayonTaskType::HashCalc => { num_threads_for_task = 2; }
    }
  } else {
    match rayon_task_type {
      RayonTaskType::ImgExtract => {
        if num_of_cpus == 1 {
          num_threads_for_task = 1;
        } else {
          num_threads_for_task = num_of_cpus - 1;
        }
      }
      RayonTaskType::HashCalc => { num_threads_for_task = 2; }
    }
  }
  num_threads_for_task
}

pub struct Services {
  notify_service: NotifyService,
  data_extraction_service: DataExtractionService,
  not_cached_books: NotCachedBooks,
  target_ext: TARGET_EXT,
  app_dirs: APP_DIRS,
  db: DB,
}
impl Services {
  pub fn new(target_ext: TARGET_EXT, app_dirs: APP_DIRS, db: DB, not_cached_books: NotCachedBooks) -> Self {
    Self {
      notify_service: NotifyService::new(target_ext.clone(), app_dirs.clone()),
      data_extraction_service: DataExtractionService::new(not_cached_books.clone(), db.clone(), app_dirs.clone()),
      not_cached_books,
      target_ext,
      app_dirs,
      db,
    }
  }
  pub fn run(&mut self, path_to_scan: String) {
    self.run_dir_scan(&path_to_scan);
    self.run_notify(path_to_scan).unwrap();
    self.run_data_extraction();
  }
  pub fn run_notify(&mut self, path_to_scan: String) -> Result<(), Error> {
    self.notify_service.run(self.db.clone(), path_to_scan)
  }
  pub fn run_data_extraction(&mut self) {
    self.data_extraction_service.run();
  }
  pub fn run_dir_scan(&self, path_to_scan: &String) {
    dir_scan_service::run(path_to_scan, &self.db, self.target_ext.clone(), self.app_dirs.clone(), &self.not_cached_books);
  }
  pub fn stop(&mut self) {
    SHUTDOWN.store(false, Ordering::Relaxed);
    self.data_extraction_service.stop();
  }
}
