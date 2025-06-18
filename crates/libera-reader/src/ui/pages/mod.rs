use concurrent_queue::ConcurrentQueue;
use gpui::{div, Context, IntoElement, ParentElement, Render, Styled, Window};
use libera_reader_core::app_dirs::AppDirs;
use libera_reader_core::db::create_db_on_disk;
use libera_reader_core::db::models::{Settings, TargetExt};
use libera_reader_core::services::Services;
use libera_reader_core::types::{NotCachedBooks, APP_DIRS, DB, SETTINGS, TARGET_EXT};
use std::sync::{Arc, RwLock};

pub(crate) mod main;
pub(crate) mod setup;
#[path = "book-viewer/mod.rs"]
pub(crate) mod book_viewer;


pub(crate) struct Pages {
  _app_dirs: APP_DIRS,
  db: DB,
  _target_ext: TARGET_EXT,
  _not_cached_books: NotCachedBooks,
  _services: Arc<Services>,
  settings: SETTINGS,
  window_of_selecting_folder_is_open: bool,
}

impl Pages {
  pub fn new() -> Self {
    let app_dirs = AppDirs::new_with_default_data_dir().unwrap();
    let db = Arc::new(create_db_on_disk(app_dirs.inn.path_to_db.clone()).unwrap());
    let target_ext = Arc::new(RwLock::new(TargetExt::new(&db)));
    let not_cached_books = NotCachedBooks::new(ConcurrentQueue::unbounded());
    let app_dirs = Arc::new(RwLock::new(app_dirs));

    let services = Arc::new(Services::new(target_ext.clone(), app_dirs.clone(), db.clone(), not_cached_books.clone()));
    let settings = Arc::new(RwLock::new(Settings::new(&db)));
    Self {
      _app_dirs: app_dirs,
      db,
      _target_ext: target_ext,
      _not_cached_books: not_cached_books,
      _services: services,
      settings,
      window_of_selecting_folder_is_open: false,
    }
  }
}

impl Render for Pages {
  fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let path_to_scan = self.settings.read().unwrap().path_to_scan.is_some();
    let setup_status = self.settings.read().unwrap().setup_is_done.clone();
    match path_to_scan && setup_status {
      true => { div().w_full().h_full().child(self.render_main_page(window, cx)) }
      false => { div().w_full().h_full().child(self.render_setup(window, cx)) }
    }
  }
}
