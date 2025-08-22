use crate::db::models::{GetOrCreate, Lang, RootRoute, SettingsModel};
use crate::db::DB;
use anyhow::Result;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Clone)]
pub struct Settings {
  inn: Arc<RwLock<SettingsModel>>,
  db: DB
}
impl Settings {
  pub fn new(db: DB) -> Result<Self> { Ok(Self { inn: Arc::new(RwLock::new(SettingsModel::get_or_create(1u32, &db)?)), db }) }
  pub fn read(&self) -> RwLockReadGuard<'_, SettingsModel> {
    self.inn.read().unwrap()
  }
  fn write(&mut self) -> RwLockWriteGuard<'_, SettingsModel> { self.inn.write().unwrap() }
  pub fn set_path_to_scan(&mut self, new_path: String) -> Result<()> {
    let path_to_scan = self.read().path_to_scan.clone();
    let old_model = self.read().clone();
    match path_to_scan {
      None => { self.write().path_to_scan = Some(new_path); }
      Some(old_path_to_scan) => {
        match old_path_to_scan.eq(&new_path) {
          true => {}
          false => {
            let mut lock = self.write();
            lock.path_to_scan = Some(new_path);
            lock.old_path_to_scan = Some(old_path_to_scan);
          }
        }
      }
    }
    self.db.update(old_model, self.read().clone())?;
    Ok(())
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
  pub fn contains_ext(&self, ext: &str) -> bool {
    let model = self.read();
    let ext_is_pdf = ext.eq("pdf") && model.pdf;
    let ext_is_epub = ext.eq("epub") && model.epub;
    let ext_is_mobi = ext.eq("mobi") && model.mobi;
    if ext_is_pdf || ext_is_epub || ext_is_mobi {
      true
    } else {
      false
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
}