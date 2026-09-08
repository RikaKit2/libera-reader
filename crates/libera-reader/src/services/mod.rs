pub mod data_extraction_service;
pub mod extraction_coordinator;
pub mod notify_service;
pub mod scan_service;

use crate::app_dirs::AppDirs;
use crate::app_ext::AppExt;
use crate::books_state::BooksState;
use crate::db::DB;
use crate::not_cached_books::{NotCachedBooks, NotCachedBooksRx};
use crate::services::scan_service::ScanService;
use crate::settings::SETTINGS;
use anyhow::Result;
pub use extraction_coordinator::{ExtractionCoordinator, ExtractionCoordinatorMode};
use gpui::App;
use notify_service::NotifyService;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkStatus {
  Working,
  NotWorking,
}

pub struct Services {
  pub notify_service: NotifyService,
  pub scan_service: ScanService,
  pub extraction_coordinator: ExtractionCoordinator,
  pub data_extraction_service_working_status: WorkStatus,
  pub extraction_rx: Option<NotCachedBooksRx>,
  settings: SETTINGS,
  db: DB,
  app_dirs: AppDirs,
  books_state: BooksState,
}

impl gpui::Global for Services {}

/// Start background scanner, watcher, and thumbnail extractor services.
pub fn start_services(cx: &mut gpui::App) {
  let scan_service = cx.services().scan_service.clone();

  cx.spawn(|async_app: &mut gpui::AsyncApp| {
    let owned_app = async_app.clone();

    async move {
      let tokio_rt = crate::TOKIO.get().unwrap();

      tokio_rt.spawn(async move {
        if let Err(e) = scan_service.run().await {
          eprintln!("ScanService error: {:?}", e);
        }
      });

      owned_app.update(|cx| {
        let _guard = tokio_rt.enter();

        let services = cx.services_mut();

        if let Err(e) = services.notify_service.run() {
          eprintln!("NotifyService error: {:?}", e);
        }

        services.run_data_extraction_service();
      });
    }
  })
  .detach();
}

impl Services {
  /// Create `Services` by pulling all required global dependencies from `App`
  /// and taking ownership of the single thumbnail extraction receiver `rx`.
  pub fn new(cx: &App, rx: NotCachedBooksRx) -> Result<Self> {
    let notify_service = NotifyService::new(cx)?;
    let scan_service = ScanService::new(cx);
    let extraction_coordinator = ExtractionCoordinator::new();

    Ok(Self {
      notify_service,
      scan_service,
      extraction_coordinator,
      data_extraction_service_working_status: WorkStatus::NotWorking,
      extraction_rx: Some(rx),
      settings: cx.settings().clone(),
      db: cx.db().clone(),
      app_dirs: cx.app_dirs().clone(),
      books_state: cx.books_state().clone(),
    })
  }

  /// Create `Services` directly from explicit dependencies (used in headless tests/benchmarks).
  pub fn from_deps(
    settings: SETTINGS, db: DB, not_cached_books: NotCachedBooks, app_dirs: AppDirs,
    books_state: BooksState, rx: NotCachedBooksRx,
  ) -> Result<Self> {
    let notify_service = NotifyService::from_deps(
      not_cached_books.clone(),
      settings.clone(),
      db.clone(),
      books_state.clone(),
    )?;
    let scan_service = ScanService::from_deps(
      settings.clone(),
      db.clone(),
      not_cached_books,
      app_dirs.clone(),
      books_state.clone(),
    );
    let extraction_coordinator = ExtractionCoordinator::new();

    Ok(Self {
      notify_service,
      scan_service,
      extraction_coordinator,
      data_extraction_service_working_status: WorkStatus::NotWorking,
      extraction_rx: Some(rx),
      settings,
      db,
      app_dirs,
      books_state,
    })
  }

  pub async fn run(&mut self) -> Result<()> {
    // Start extraction service FIRST so it's already running when books arrive
    self.run_data_extraction_service();

    self.scan_service.run().await?;
    self.db.compact()?;
    self.notify_service.run()?;
    Ok(())
  }

  pub fn run_data_extraction_service(&mut self) {
    match self.data_extraction_service_working_status {
      WorkStatus::Working => {}
      WorkStatus::NotWorking => {
        if let Some(rx) = self.extraction_rx.take() {
          let db = self.db.clone();
          let app_dirs = self.app_dirs.clone();
          let books_state = self.books_state.clone();
          let settings = self.settings.clone();
          let coordinator_rx = self.extraction_coordinator.subscribe();

          tokio::spawn(async move {
            data_extraction_service::run(db, app_dirs, rx, books_state, settings, coordinator_rx)
              .await;
          });

          self.data_extraction_service_working_status = WorkStatus::Working;
        }
      }
    }
  }

  pub fn stop(&mut self) -> Result<()> {
    self.notify_service.stop()?;
    Ok(())
  }
}
