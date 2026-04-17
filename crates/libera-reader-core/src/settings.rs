use crate::db::DB;
use crate::db::models::books::book::BookExt;
use crate::db::models::{AppTheme, GetOrCreate, Lang, RootRoute, Settings};
use anyhow::Result;
use std::path::PathBuf;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Clone)]
pub struct SETTINGS {
  inn: Arc<RwLock<Settings>>,
  db: DB,
}
impl SETTINGS {
  pub fn new(db: DB) -> Result<Self> {
    Ok(Self { inn: Arc::new(RwLock::new(Settings::get_or_create(1u32, &db)?)), db })
  }
  pub fn read(&self) -> RwLockReadGuard<'_, Settings> {
    self.inn.read().unwrap()
  }
  fn write(&mut self) -> RwLockWriteGuard<'_, Settings> {
    self.inn.write().unwrap()
  }
  pub fn set_path_to_scan(&mut self, new_path: PathBuf) -> Result<()> {
    let path_to_scan = self.read().path_to_scan.clone();
    let old_model = self.read().clone();
    match path_to_scan {
      None => {
        self.write().path_to_scan = Some(new_path);
      }
      Some(previous_path_to_scan) => match previous_path_to_scan.eq(&new_path) {
        true => {}
        false => {
          let mut lock = self.write();
          lock.path_to_scan = Some(new_path);
          lock.previous_path_to_scan = Some(previous_path_to_scan);
        }
      },
    }
    self.db.update(old_model, self.read().clone())?;
    Ok(())
  }
  pub fn get_path_to_scan_str(&self) -> Option<String> {
    let guard = self.read();
    guard.path_to_scan.as_ref().map(|path| path.to_string_lossy().to_string())
  }
  pub fn get_path_to_scan_if_exists(&self) -> Option<PathBuf> {
    let guard = self.read();
    match &guard.path_to_scan {
      None => None,
      Some(path_to_scan) => match path_to_scan.exists() {
        true => Some(path_to_scan.clone()),
        false => None,
      },
    }
  }
  pub fn set_language(&mut self, lang: Lang) -> Result<()> {
    let old_model = self.read().clone();
    match old_model.language.eq(&lang) {
      true => {}
      false => {
        self.write().language = lang;
        self.db.update(old_model, self.read().clone())?;
      }
    };
    Ok(())
  }
  pub fn set_theme(&mut self, new_theme: &AppTheme) -> Result<()> {
    let old_model = self.read().clone();
    if &old_model.theme != new_theme {
      self.write().theme = new_theme.clone();
      self.db.update(old_model, self.read().clone())?;
    }
    Ok(())
  }
  pub fn set_route(&mut self, new_route: RootRoute) -> Result<()> {
    let old_model = self.read().clone();
    match old_model.route.eq(&new_route) {
      true => {}
      false => {
        self.write().route = new_route;
        self.db.update(old_model, self.read().clone())?;
      }
    }
    Ok(())
  }
  pub fn set_setup_status(&mut self, status: bool) -> Result<()> {
    let old_model = self.read().clone();
    match old_model.setup_is_done.eq(&status) {
      true => {}
      false => {
        self.write().setup_is_done = status;
        self.db.update(old_model, self.read().clone())?;
      }
    }
    Ok(())
  }
  pub fn contains_ext(&self, ext: &BookExt) -> bool {
    let model = self.read();
    match ext {
      BookExt::PDF(_) => model.pdf,
      BookExt::EPUB(_) => model.epub,
      BookExt::MOBI(_) => model.mobi,
    }
  }
  pub fn invert_pdf(&mut self) -> Result<()> {
    let old_model = self.read().clone();
    self.write().pdf = !old_model.pdf;
    self.db.update(old_model, self.read().clone())?;
    Ok(())
  }
  pub fn invert_epub(&mut self) -> Result<()> {
    let old_model = self.read().clone();
    self.write().epub = !old_model.epub;
    self.db.update(old_model, self.read().clone())?;
    Ok(())
  }
  pub fn invert_mobi(&mut self) -> Result<()> {
    let old_model = self.read().clone();
    self.write().mobi = !old_model.mobi;
    self.db.update(old_model, self.read().clone())?;
    Ok(())
  }
  pub fn set_number_of_columns(&mut self, columns: u32) -> Result<()> {
    let old_model = self.read().clone();
    if old_model.number_of_columns != columns {
      self.write().number_of_columns = columns;
      self.db.update(old_model, self.read().clone())?;
    }
    Ok(())
  }
  pub fn to_next_setup_route(&mut self) {
    let current_route = self.read().route;
    if let RootRoute::Setup(old_route) = current_route
      && let Some(new_route) = old_route.next()
    {
      let old_model = self.read().clone();
      self.write().route = RootRoute::Setup(new_route);
      self.db.update(old_model, self.read().clone()).unwrap();
    }
  }
  pub fn to_previous_setup_route(&mut self) {
    let current_route = self.read().route;
    if let RootRoute::Setup(old_route) = current_route
      && let Some(new_route) = old_route.back()
    {
      let old_model = self.read().clone();
      self.write().route = RootRoute::Setup(new_route);
      self.db.update(old_model, self.read().clone()).unwrap();
    };
  }
}
