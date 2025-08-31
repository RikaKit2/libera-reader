use crate::app_dirs::AppDirs;
use crate::db::DB;
use crate::db::models::GetText;
use crate::services::SERVICES;
use crate::settings::Settings;
use anyhow::Result;
use gpui::{App, Global};
use std::path::PathBuf;

pub struct Ctx {
  pub settings: Settings,
  pub services: SERVICES,
  pub app_dirs: AppDirs,
  pub db: DB,
}
impl Ctx {
  pub fn new() -> Result<Self> {
    let app_dirs = AppDirs::new_with_default_data_dir().unwrap();
    let db = DB::new(app_dirs.read().path_to_db.clone())?;
    let settings = Settings::new(db.clone())?;
    Ok(Self { services: SERVICES::new(settings.clone(), app_dirs.clone(), db.clone())?, settings, app_dirs, db })
  }
  pub fn new_for_test(path_to_data_dir: PathBuf) -> Result<Self> {
    let app_dirs = AppDirs::new(path_to_data_dir).unwrap();
    let db = DB::new(app_dirs.read().path_to_db.clone())?;
    let settings = Settings::new(db.clone())?;
    Ok(Self { services: SERVICES::new(settings.clone(), app_dirs.clone(), db.clone())?, settings, app_dirs, db })
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
}

impl Global for Ctx {}
pub trait GlobalCTX {
  fn ctx(&self) -> &Ctx;
  fn ctx_mut(&mut self) -> &mut Ctx;
  fn i18n<T: GetText>(&self, target_enum: T) -> &'static str;
}
impl GlobalCTX for App {
  fn ctx(&self) -> &Ctx {
    Ctx::global(self)
  }
  fn ctx_mut(&mut self) -> &mut Ctx {
    Ctx::global_mut(self)
  }
  fn i18n<T: GetText>(&self, target_enum: T) -> &'static str {
    self.ctx().settings.read().language.get(target_enum)
  }
}
