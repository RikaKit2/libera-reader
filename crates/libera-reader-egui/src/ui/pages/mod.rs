use crate::router::{RootRoute, Route, Router};
use crate::ui::pages::book_viewer::BookViewer;
use crate::ui::pages::main::Main;
use crate::ui::pages::setup::Setup;
use concurrent_queue::ConcurrentQueue;
use egui::Context;
use libera_reader_core::app_dirs::AppDirs;
use libera_reader_core::db::create_db_on_disk;
use libera_reader_core::db::models::{Settings, TargetExt};
use libera_reader_core::services::Services;
use libera_reader_core::types::{AppDirsType, NotCachedBooks, TypeTargetExt, DB};
use std::sync::{Arc, RwLock};

pub(crate) mod main;
pub(crate) mod setup;
#[path = "book-viewer/mod.rs"]
pub(crate) mod book_viewer;

pub(crate) struct Pages {
  main: Main,
  setup: Setup,
  book_viewer: BookViewer,
  router: Router,

  app_dirs: AppDirsType,
  db: DB,
  target_ext: TypeTargetExt,
  not_cached_books: NotCachedBooks,
  services: Services,
  settings: Settings,
}
impl Pages {
  pub fn new() -> Self {
    let app_dirs = AppDirs::new_with_default_data_dir().unwrap();
    let db = Arc::new(create_db_on_disk(app_dirs.inn.path_to_db.clone()).unwrap());
    let target_ext = Arc::new(RwLock::new(TargetExt::new(&db)));
    let not_cached_books = NotCachedBooks::new(ConcurrentQueue::unbounded());
    let app_dirs = Arc::new(RwLock::new(app_dirs));
    let services = Services::new(target_ext.clone(), app_dirs.clone(), db.clone(), not_cached_books.clone());
    let settings = Settings::new(&db);
    let mut router = Router::new(RootRoute::Setup);
    match settings.path_to_scan.is_some() {
      true => { router.set_route(RootRoute::Base(Route::Library)) }
      false => {}
    }
    Self {
      main: Main::new(),
      setup: Setup::new(),
      book_viewer: BookViewer::new(),
      router,
      app_dirs,
      db,
      target_ext,
      not_cached_books,
      services,
      settings,
    }
  }
  pub(crate) fn make(&mut self, ctx: &Context) {
    match &self.router.inn {
      RootRoute::Base(_) => { self.main.make(ctx, &mut self.router); }
      RootRoute::BookViewer => { self.book_viewer.make(ctx); }
      RootRoute::Setup => { self.setup.make(ctx, &mut self.settings, &self.db, &mut self.router); }
    };
  }
}
