mod notify_service;
pub(crate) mod scan_service;
use anyhow::Result;
use notify_service::NotifyService;

use crate::{db::DB, error_handler::ErrorHandler, not_cached_books::NotCachedBooks, services::scan_service::ScanService, settings::SETTINGS};

pub enum WorkStatus {
  Working,
  NotWorking,
}

pub struct Services {
  pub notify_service: NotifyService,
  pub scan_service: ScanService,
  db: DB,
}
impl Services {
  pub(crate) fn new(settings: SETTINGS, db: DB, not_cached_books: NotCachedBooks, error_handler: ErrorHandler) -> Result<Self> {
    let notify_service = NotifyService::new(not_cached_books.clone(), settings.clone(), db.clone())?;
    let scan_service = ScanService::new(not_cached_books, settings, db.clone(), error_handler);
    Ok(Self { notify_service, scan_service, db })
  }
  pub fn run(&mut self) -> Result<()> {
    self.scan_service.run()?;
    self.db.compact()?;
    self.db.save_to_storage()?;
    self.db.reload_db()?;
    // self.notify_service.run()?;
    Ok(())
  }
  pub fn stop(&mut self) -> Result<()> {
    self.notify_service.stop()?;
    Ok(())
  }
}
