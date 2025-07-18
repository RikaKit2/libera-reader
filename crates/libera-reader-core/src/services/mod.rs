use crate::services::notify_service::NotifyEventHandler;
use crate::types::{NotCachedBooks, APP_DIRS, DB, TARGET_EXT};
use anyhow::Result;
use notify::RecommendedWatcher;

mod notify_service;
mod passive_scan_service;

pub enum Status {
  Working,
  NotWorking,
}

pub struct Services {
  path_to_scan: Option<String>,
  watcher: RecommendedWatcher,
  not_cached_books: NotCachedBooks,
  target_ext: TARGET_EXT,
  app_dirs: APP_DIRS,
  db: DB,
  // data_extraction_service: DataExtractionService,
}
impl Services {
  pub fn new(not_cached_books: NotCachedBooks, target_ext: TARGET_EXT, app_dirs: APP_DIRS, db: DB) -> Result<Self> {
    Ok(Self {
      path_to_scan: None,
      watcher: notify::recommended_watcher(NotifyEventHandler::new(not_cached_books.clone(), target_ext.clone(), app_dirs.clone(), db.clone()))?,
      // data_extraction_service: DataExtractionService::new(not_cached_books.clone(), db.clone(), app_dirs.clone()),
      not_cached_books,
      target_ext,
      app_dirs,
      db,
    })
  }
  pub fn run(&mut self, path_to_scan: String) -> Result<()> {
    self.run_passive_scan(&path_to_scan)?;
    self.run_notify(path_to_scan)?;
    Ok(())
  }
  pub fn stop(&mut self) -> Result<()> {
    self.stop_notify()?;
    Ok(())
  }
}
