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
pub struct Services {
  data_extraction_service_working_status: Status,
  notify_service_working_status: Status,
  not_cached_books: NotCachedBooks,
  settings: Settings,
  watcher: RecommendedWatcher,
  app_dirs: APP_DIRS,
  db: DB,
}

impl Services {
  pub fn new(settings: Settings, app_dirs: APP_DIRS, db: DB) -> Result<Self> {
    let not_cached_books = NotCachedBooks::new(ConcurrentQueue::unbounded());
    Ok(Self {
      data_extraction_service_working_status: Status::NotWorking,
      notify_service_working_status: Status::NotWorking,
      watcher: notify::recommended_watcher(NotifyEventHandler::new(settings.clone(), not_cached_books.clone(), db.clone()))?,
      not_cached_books,
      settings,
      app_dirs,
      db,
    })
  }
}
pub struct SERVICES {
  inn: Arc<RwLock<Services>>,
  worker: Option<thread::JoinHandle<()>>,
}
impl SERVICES {
  pub fn new(settings: Settings, app_dirs: APP_DIRS, db: DB) -> Result<Self> {
    Ok(Self { inn: Arc::new(RwLock::new(Services::new(settings, app_dirs, db)?)), worker: None })
  }
  pub fn run(&mut self) -> Result<()> {
    match &self.worker {
      None => {
        let services = self.inn.clone();
        let worker = thread::spawn(move || {
          let rt = Builder::new_multi_thread().enable_all().build().unwrap();
          rt.block_on(async {
            let mut lock = services.write().unwrap();
            lock.run_passive_scan().unwrap();
            lock.db.compact().unwrap();
            lock.db.save_to_storage().unwrap();
            lock.db.reload_db().unwrap();
            lock.run_notify().unwrap();
            match lock.run_data_extraction_service().await {
              None => {}
              Some(j) => { j.await.unwrap(); }
            };
          });
        });
        self.worker = Some(worker);
      }
      Some(j) => {
        match j.is_finished() {
          true => { 
            self.worker = None;
            self.run()?
          }
          false => {}
        }
      }
    }
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
