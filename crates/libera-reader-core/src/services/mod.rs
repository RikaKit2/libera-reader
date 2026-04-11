pub mod notify_service;
pub mod scan_service;
use anyhow::Result;
use notify_service::NotifyService;

use crate::types::LibraryEvent;
use crate::{db::DB, not_cached_books::NotCachedBooks, services::scan_service::ScanService, settings::SETTINGS};
use tokio::sync::broadcast;

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
  pub(crate) fn new(
    settings: SETTINGS, db: DB, not_cached_books: NotCachedBooks, event_tx: broadcast::Sender<LibraryEvent>,
  ) -> Result<Self> {
    let notify_service =
      NotifyService::new(not_cached_books.clone(), settings.clone(), db.clone(), event_tx.clone())?;
    let scan_service = ScanService::new(settings, db.clone(), event_tx);
    Ok(Self { notify_service, scan_service, db })
  }
  pub async fn run(&mut self) -> Result<()> {
    self.scan_service.run().await?;
    self.db.compact()?;
    self.notify_service.run()?;
    Ok(())
  }
  pub fn stop(&mut self) -> Result<()> {
    self.notify_service.stop()?;
    Ok(())
  }
}
