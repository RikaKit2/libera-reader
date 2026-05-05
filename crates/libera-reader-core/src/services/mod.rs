pub mod data_extraction_service;
pub mod notify_service;
pub mod scan_service;

use anyhow::Result;
use notify_service::NotifyService;

use crate::types::LibraryEvent;
use crate::{
  db::DB, not_cached_books::NotCachedBooks, services::scan_service::ScanService, settings::SETTINGS,
};
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkStatus {
  Working,
  NotWorking,
}

pub struct Services {
  pub notify_service: NotifyService,
  pub scan_service: ScanService,
  pub data_extraction_service_working_status: WorkStatus,
  pub not_cached_books: NotCachedBooks,
  db: DB,
  app_dirs: crate::app_dirs::AppDirs,
  event_tx: broadcast::Sender<LibraryEvent>,
}

impl Services {
  pub(crate) fn new(
    settings: SETTINGS, db: DB, not_cached_books: NotCachedBooks,
    event_tx: broadcast::Sender<LibraryEvent>, app_dirs: crate::app_dirs::AppDirs,
  ) -> Result<Self> {
    let notify_service =
      NotifyService::new(not_cached_books.clone(), settings.clone(), db.clone(), event_tx.clone())?;
    let scan_service = ScanService::new(settings, db.clone(), event_tx.clone());

    Ok(Self {
      notify_service,
      scan_service,
      data_extraction_service_working_status: WorkStatus::NotWorking,
      not_cached_books,
      db,
      app_dirs,
      event_tx,
    })
  }

  pub async fn run(&mut self) -> Result<()> {
    self.scan_service.run().await?;
    self.db.compact()?;
    self.notify_service.run()?;
    // Start the thumbnail extraction service
    self.run_data_extraction_service();
    Ok(())
  }

  pub fn run_data_extraction_service(&mut self) {
    if self.data_extraction_service_working_status == WorkStatus::NotWorking {
      let db = self.db.clone();
      let app_dirs = self.app_dirs.clone();
      let not_cached_books = self.not_cached_books.clone();
      let event_tx = self.event_tx.clone();

      if let (Some(high_rx), Some(low_rx)) =
        (not_cached_books.take_high_rx(), not_cached_books.take_low_rx())
      {
        let processing_now = Arc::clone(not_cached_books.processing_now());

        tokio::spawn(async move {
          data_extraction_service::run_data_extraction_service(
            db,
            app_dirs,
            high_rx,
            low_rx,
            processing_now,
            event_tx,
          )
          .await;
        });

        self.data_extraction_service_working_status = WorkStatus::Working;
      }
    }
  }

  pub fn stop(&mut self) -> Result<()> {
    self.notify_service.stop()?;
    Ok(())
  }
}
