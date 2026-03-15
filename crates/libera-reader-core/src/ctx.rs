use crate::db::DB;
use crate::db::models::{AppTheme, Lang, RootRoute};
use crate::error_handler::{ErrorHandler, ErrorReceiver};
use crate::services::Services;
use crate::settings::SETTINGS;
use crate::{app_dirs::AppDirs, not_cached_books::NotCachedBooks};
use anyhow::Result;
use gpui::{App, Global};
use std::path::PathBuf;
use utils::debug;

pub struct Ctx {
  pub settings: SETTINGS,
  pub services: Services,
  pub app_dirs: AppDirs,
  pub not_cached_books: NotCachedBooks,
  pub error_handler: ErrorHandler,
  pub error_receiver: ErrorReceiver,
  pub db: DB,
}
impl Ctx {
  pub fn new() -> Result<Self> {
    let app_dirs = AppDirs::new_with_default_data_dir().unwrap();
    Self::base_new(app_dirs)
  }
  pub fn new_for_test(path_to_data_dir: PathBuf) -> Result<Self> {
    let app_dirs = AppDirs::new(path_to_data_dir).unwrap();
    Self::base_new(app_dirs)
  }
  fn base_new(app_dirs: AppDirs) -> Result<Self> {
    let (error_handler, error_receiver) = ErrorHandler::new();
    let path_to_db = app_dirs.read().path_to_db.clone();
    debug!("Path to db: {:?}", &path_to_db);
    debug!("DB exists: {:?}", &path_to_db.exists());
    let db = DB::new(path_to_db)?;
    let settings = SETTINGS::new(db.clone())?;
    let not_cached_books = NotCachedBooks::new();
    let services = Services::new(settings.clone(), db.clone(), not_cached_books.clone(), error_handler.clone())?;
    Ok(Self { services, settings, app_dirs, not_cached_books, db, error_handler, error_receiver })
  }
  pub fn init(cx: &mut App) {
    cx.set_global::<Self>(Self::new().unwrap())
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
