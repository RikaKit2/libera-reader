use crate::db::DB;
use crate::db::models::BookMark;
use crate::db::models::books::book::BookPath;

use crate::app_dirs::AppDirs;
use crate::db::models::{AppTheme, Lang, RootRoute};
use crate::error_handler::{ErrorHandler, ErrorReceiver};
use crate::not_cached_books::NotCachedBooks;
use crate::send_event;
use crate::services::Services;
use crate::settings::SETTINGS;
use crate::types::LibraryEvent;
use gpui::{App, Global};
use std::path::PathBuf;
use tokio::sync::broadcast;
use utils::debug;

pub type LibraryEventSender = broadcast::Sender<LibraryEvent>;

pub struct Ctx {
  pub settings: SETTINGS,
  pub services: Services,
  pub app_dirs: AppDirs,
  pub not_cached_books: NotCachedBooks,
  pub error_handler: ErrorHandler,
  pub error_receiver: ErrorReceiver,
  pub db: DB,
  pub event_tx: LibraryEventSender,
}
impl Default for Ctx {
  fn default() -> Self {
    Self::new()
  }
}

impl Ctx {
  pub fn new() -> Self {
    let app_dirs = AppDirs::new_with_default_data_dir().unwrap();
    Self::base_new(app_dirs)
  }
  pub fn new_for_test(path_to_data_dir: PathBuf) -> Self {
    let app_dirs = AppDirs::new(path_to_data_dir).unwrap();
    Self::base_new(app_dirs)
  }
  fn base_new(app_dirs: AppDirs) -> Self {
    let (error_handler, error_receiver) = ErrorHandler::new();
    let path_to_db = app_dirs.read().path_to_db.clone();
    debug!("Path to db: {:?}", &path_to_db);
    debug!("DB exists: {:?}", &path_to_db.exists());
    let db = DB::new(path_to_db).unwrap();
    let settings = SETTINGS::new(db.clone()).unwrap();
    let not_cached_books = NotCachedBooks::new();
    let (event_tx, _) = broadcast::channel::<LibraryEvent>(1024);
    let services = Services::new(
      settings.clone(),
      db.clone(),
      not_cached_books.clone(),
      event_tx.clone(),
      app_dirs.clone(),
    )
    .unwrap();
    Self {
      services,
      settings,
      app_dirs,
      not_cached_books,
      db,
      error_handler,
      error_receiver,
      event_tx,
    }
  }
  pub fn init(cx: &mut App) {
    cx.set_global::<Self>(Self::new())
  }
  #[inline(always)]
  pub fn global(cx: &App) -> &Self {
    cx.global::<Self>()
  }
  #[inline(always)]
  pub fn global_mut(cx: &mut App) -> &mut Self {
    cx.global_mut::<Self>()
  }
  pub fn theme(&self) -> AppTheme {
    self.settings.read().theme.clone()
  }
  pub fn lang(&self) -> Lang {
    self.settings.read().language.clone()
  }
  pub fn get_curr_route(&self) -> RootRoute {
    self.settings.read().route
  }

  pub fn add_bookmark(&self, book_path: BookPath, bookmark: BookMark) -> anyhow::Result<()> {
    if let Some(updated_book) = self.db.add_bookmark(book_path.clone(), bookmark)? {
      let snapshot = self.snapshot_for(&updated_book);
      send_event!(self.event_tx, LibraryEvent::BookUpdated(snapshot));
      send_event!(self.event_tx, LibraryEvent::BookMarkAdded { book_path });
    }
    Ok(())
  }

  pub fn update_bookmark(&self, book_path: BookPath, bookmark: BookMark) -> anyhow::Result<()> {
    if let Some(updated_book) = self.db.update_bookmark(book_path.clone(), bookmark)? {
      let snapshot = self.snapshot_for(&updated_book);
      send_event!(self.event_tx, LibraryEvent::BookUpdated(snapshot));
      send_event!(self.event_tx, LibraryEvent::BookMarkUpdated { book_path });
    }
    Ok(())
  }

  pub fn remove_bookmark(&self, book_path: BookPath, time_created: &str) -> anyhow::Result<()> {
    if let Some(updated_book) = self.db.remove_bookmark(book_path.clone(), time_created)? {
      let snapshot = self.snapshot_for(&updated_book);
      send_event!(self.event_tx, LibraryEvent::BookUpdated(snapshot));
      send_event!(self.event_tx, LibraryEvent::BookMarkRemoved { book_path });
    }
    Ok(())
  }

  /// Build a UI-ready snapshot, consulting the filesystem for `has_thumbnail`.
  /// This replaces the old cached `Book.has_thumbnail` field and is the same
  /// source of truth used by the UI's `ThumbnailCache`.
  fn snapshot_for(
    &self, book: &crate::db::models::books::book::Book,
  ) -> crate::db::models::books::book::BookSnapshot {
    let thumbnails_dir = self.app_dirs.read().thumbnails_dir.clone();
    crate::db::models::books::book::BookSnapshot::from_book(
      book,
      book.has_thumbnail_on_disk(&self.db, &thumbnails_dir),
    )
  }
}

impl Global for Ctx {}
pub trait GlobalCTX {
  fn ctx(&self) -> &Ctx;
  fn ctx_mut(&mut self) -> &mut Ctx;
}
impl GlobalCTX for App {
  fn ctx(&self) -> &Ctx {
    Ctx::global(self)
  }
  fn ctx_mut(&mut self) -> &mut Ctx {
    Ctx::global_mut(self)
  }
}
