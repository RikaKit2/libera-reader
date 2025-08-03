use crate::app_dirs::AppDirs;
use crate::db::models::GetText;
use crate::db::{create_db_in_memory, create_db_on_disk};
use crate::services::SERVICES;
use crate::settings::Settings;
use crate::types::{APP_DIRS, DB};
use anyhow::Result;
use gpui::{App, Global};
use native_db::Models;
use std::path::PathBuf;
use std::sync::Arc;

pub struct Ctx {
  pub settings: Settings,
  pub services: SERVICES,
  pub app_dirs: APP_DIRS,
  pub db: DB,
}
impl Ctx {
  pub fn new(models: &'static Models) -> Result<Self> {
    let app_dirs = AppDirs::new_with_default_data_dir().unwrap();
    let db: DB = Arc::new(create_db_on_disk(app_dirs.read().path_to_db.clone(), models)?);

    let app_dirs = Arc::new(app_dirs);
    let settings = Settings::new(db.clone())?;
    Ok(Self {
      services: SERVICES::new(settings.clone(), app_dirs.clone(), db.clone())?,
      settings,
      app_dirs,
      db,
    })
  }
  pub fn new_for_test(models: &'static Models, path_to_data_dir: PathBuf) -> Result<Self> {
    let app_dirs = AppDirs::new(path_to_data_dir).unwrap();
    let db = Arc::new(create_db_in_memory(models)?);

    let app_dirs = Arc::new(app_dirs);
    let settings = Settings::new(db.clone())?;
    Ok(Self {
      services: SERVICES::new(settings.clone(), app_dirs.clone(), db.clone())?,
      settings,
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
    self.ctx().settings.read().language.get(target_enum)
  }
}
