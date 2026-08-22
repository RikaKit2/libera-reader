pub mod data_extraction_service;
pub mod notify_service;
pub mod scan_service;

use crate::app_dirs::AppDirs;
use crate::app_ext::AppExt;
use crate::books_state::BooksState;
use crate::{
  db::DB, not_cached_books::NotCachedBooks, services::scan_service::ScanService, settings::SETTINGS,
};
use anyhow::Result;
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
  pub data_extraction_service_working_status: WorkStatus,
  pub not_cached_books: NotCachedBooks,
  db: DB,
  app_dirs: AppDirs,
  books_state: BooksState,
}

impl gpui::Global for Services {}

/// Start background scanner, watcher, and thumbnail extractor services.
pub fn start_services(cx: &mut gpui::App) {
  let scan_service = cx.global::<Services>().scan_service.clone();

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

        let services = cx.global_mut::<Services>();

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
  /// Create `Services` by pulling all required global dependencies from `App`.
  pub fn new(cx: &App) -> Result<Self> {
    Self::from_deps(
      cx.settings().clone(),
      cx.db().clone(),
      cx.not_cached_books().clone(),
      cx.app_dirs().clone(),
      cx.books_state().clone(),
    )
  }

  /// Create `Services` directly from explicit dependencies (used in headless tests/benchmarks).
  pub fn from_deps(
    settings: SETTINGS, db: DB, not_cached_books: NotCachedBooks, app_dirs: AppDirs,
    books_state: BooksState,
  ) -> Result<Self> {
    let notify_service = NotifyService::new(
      not_cached_books.clone(),
      settings.clone(),
      db.clone(),
      app_dirs.clone(),
      books_state.clone(),
    )?;
    let scan_service = ScanService::new(
      settings,
      db.clone(),
      not_cached_books.clone(),
      app_dirs.clone(),
      books_state.clone(),
    );

    Ok(Self {
      notify_service,
      scan_service,
      data_extraction_service_working_status: WorkStatus::NotWorking,
      not_cached_books,
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
    if self.data_extraction_service_working_status == WorkStatus::NotWorking {
      let db = self.db.clone();
      let app_dirs = self.app_dirs.clone();
      let books_state = self.books_state.clone();
      let settings = self.scan_service.settings().clone();

      if let Some(rx) = self.not_cached_books.take_rx() {
        tokio::spawn(async move {
          data_extraction_service::run(db, app_dirs, rx, books_state, settings).await;
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
