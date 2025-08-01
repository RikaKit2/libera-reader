use crate::services::notify_service::NotifyEventHandler;
use crate::settings::Settings;
use crate::types::{NotCachedBooks, APP_DIRS, DB};
use anyhow::Result;
use concurrent_queue::ConcurrentQueue;
use notify::RecommendedWatcher;
use std::sync::{Arc, RwLock};
use std::thread;
use tokio::runtime::Builder;

mod notify_service;
mod passive_scan_service;
mod data_extraction_service;

pub enum Status {
  Working,
  NotWorking,
}
pub struct State {
  pub data_extraction_service_working_status: Status,
  pub notify_service_working_status: Status,
  not_cached_books: NotCachedBooks,
  settings: Settings,
  watcher: RecommendedWatcher,
  app_dirs: APP_DIRS,
  db: DB,
}

impl State {
  pub fn new(settings: Settings, app_dirs: APP_DIRS, db: DB) -> Result<Self> {
    Ok(Self {
      data_extraction_service_working_status: Status::NotWorking,
      notify_service_working_status: Status::NotWorking,
      watcher: notify::recommended_watcher(NotifyEventHandler::new(settings.clone(), db.clone()))?,
      not_cached_books: NotCachedBooks::new(ConcurrentQueue::unbounded()),
      settings,
      app_dirs,
      db,
    })
  }
}
pub struct SERVICES {
  inn: Arc<RwLock<State>>
}
impl SERVICES {
  pub fn new(settings: Settings, app_dirs: APP_DIRS, db: DB) -> Result<Self> {
    Ok(Self { inn: Arc::new(RwLock::new(State::new(settings, app_dirs, db)?)) })
  }
  pub fn run(&self) -> Result<()> {
    let services = self.inn.clone();
    thread::spawn(move || {
      let rt = Builder::new_current_thread().enable_all().build().unwrap();
      rt.block_on(async {
        services.write().unwrap().run_passive_scan().unwrap();
        services.write().unwrap().run_notify().unwrap();
        // services.write().unwrap().run_data_extraction_service().await;
      });
    });
    Ok(())
  }
  pub fn run_passive_scan(&mut self) -> Result<()> {
    self.inn.write().unwrap().run_passive_scan()?;
    Ok(())
  }
  pub fn run_notify(&mut self) -> Result<()> {
    self.inn.write().unwrap().run_notify()?;
    Ok(())
  }
  pub fn stop(&mut self) -> Result<()> {
    self.inn.write().unwrap().stop_notify()?;
    Ok(())
  }
}
