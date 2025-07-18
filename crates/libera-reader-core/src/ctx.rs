use crate::app_dirs::AppDirs;
use crate::db::models::{GetText, TargetExt};
use crate::db::{create_db_in_memory, create_db_on_disk};
use crate::services::Services;
use crate::settings::SETTINGS;
use crate::types::{NotCachedBooks, APP_DIRS, DB, TARGET_EXT};
use anyhow::Result;
use concurrent_queue::ConcurrentQueue;
use gpui::{App, Global};
use native_db::Models;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

pub struct Ctx {
  pub not_cached_books: NotCachedBooks,
  pub target_ext: TARGET_EXT,
  pub settings: SETTINGS,
  pub services: Services,
  pub app_dirs: APP_DIRS,
  pub db: DB,
}
impl Ctx {
  pub fn new(models: &'static Models) -> Result<Self> {
    let app_dirs = AppDirs::new_with_default_data_dir().unwrap();
    let db = Arc::new(create_db_on_disk(app_dirs.inn.path_to_db.clone(), models)?);

    let not_cached_books = NotCachedBooks::new(ConcurrentQueue::unbounded());
    let target_ext = Arc::new(RwLock::new(TargetExt::new(&db)?));
    let app_dirs = Arc::new(RwLock::new(app_dirs));
    Ok(Self {
      not_cached_books: not_cached_books.clone(),
      target_ext: target_ext.clone(),
      services: Services::new(not_cached_books, target_ext, app_dirs.clone(), db.clone())?,
      settings: SETTINGS::new(db.clone())?,
      app_dirs,
      db,
    })
  }
  pub fn new_for_test(models: &'static Models, path_to_data_dir: PathBuf) -> Result<Self> {
    let app_dirs = AppDirs::new(path_to_data_dir).unwrap();
    let db = Arc::new(create_db_in_memory(models)?);

    let not_cached_books = NotCachedBooks::new(ConcurrentQueue::unbounded());
    let target_ext = Arc::new(RwLock::new(TargetExt::new(&db)?));
    let app_dirs = Arc::new(RwLock::new(app_dirs));
    Ok(Self {
      not_cached_books: not_cached_books.clone(),
      target_ext: target_ext.clone(),
      services: Services::new(not_cached_books, target_ext, app_dirs.clone(), db.clone())?,
      settings: SETTINGS::new(db.clone())?,
      app_dirs,
      db,
    })
  }
  pub fn init(cx: &mut App, models: &'static Models) {
    cx.set_global::<Self>(Self::new(models).unwrap())
  }

  #[inline(always)]
  pub fn global(cx: &App) -> &Self {
    cx.global::<Self>()
  }

  #[inline(always)]
  pub fn global_mut(cx: &mut App) -> &mut Self {
    cx.global_mut::<Self>()
  }
}

impl Global for Ctx {}
pub trait GlobalCTX {
  fn ctx(&self) -> &Ctx;
  fn ctx_mut(&mut self) -> &mut Ctx;
  fn i18n<T: GetText>(&self, target_enum: T) -> &'static str;
}
impl GlobalCTX for App {
  fn ctx(&self) -> &Ctx { Ctx::global(self) }
  fn ctx_mut(&mut self) -> &mut Ctx {
    Ctx::global_mut(self)
  }
  fn i18n<T: GetText>(&self, target_enum: T) -> &'static str {
    self.ctx().settings.language.get(target_enum)
  }
}
