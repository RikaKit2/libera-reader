use crate::db::crud::update_table;
use crate::db::models::{GetOrCreate, Lang, RootRoute, SettingsModel};
use crate::types::DB;
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
  pub fn write(&mut self) -> RwLockWriteGuard<'_, SettingsModel> { self.inn.write().unwrap() }
  pub fn set_path_to_scan(&mut self, new_path: String) -> Result<()> {
    let flag = match &self.read().path_to_scan {
      None => { true }
      Some(old_path) => { old_path.eq(&new_path) }
    };
    match flag {
      true => {
        update_table(&self.db, Some(self.read().clone()), |model| model.path_to_scan = Some(new_path.clone()))?;
        self.write().path_to_scan = Some(new_path);
      }
      false => {}
    }
    Ok(())
  }
  pub fn set_language(&mut self, lang: Lang) -> Result<()> {
    let lang_eq = self.read().language.eq(&lang);
    match lang_eq {
      true => {}
      false => {
        update_table(&self.db, Some(self.read().clone()), |model| model.language = lang.clone())?;
        self.write().language = lang;
      }
    };
    Ok(())
  }
  pub fn set_route(&mut self, new_route: RootRoute) -> Result<()> {
    let route_eq = self.read().route.eq(&new_route);
    match route_eq {
      true => {}
      false => {
        update_table(&self.db, Some(self.read().clone()), |model| model.route = new_route.clone())?;
        self.write().route = new_route;
      }
    }
    Ok(())
  }
  pub fn set_setup_status(&mut self, status: bool) -> Result<()> {
    let setup_eq = self.read().setup_is_done.eq(&status);
    match setup_eq {
      true => {}
      false => {
        update_table(&self.db, Some(self.read().clone()), |model| model.setup_is_done = status)?;
        self.write().setup_is_done = status;
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
    update_table(&self.db, Some(self.read().clone()), |target_ext| target_ext.pdf = !target_ext.pdf)?;
    let old_value = self.read().pdf;
    self.write().pdf = !old_value;
    Ok(())
  }
  pub fn invert_epub(&mut self) -> Result<()> {
    update_table(&self.db, Some(self.read().clone()), |target_ext| target_ext.epub = !target_ext.epub)?;
    let old_value = self.read().epub;
    self.write().epub = !old_value;
    Ok(())
  }
  pub fn invert_mobi(&mut self) -> Result<()> {
    update_table(&self.db, Some(self.read().clone()), |target_ext| target_ext.mobi = !target_ext.mobi)?;
    let old_value = self.read().mobi;
    self.write().mobi = !old_value;
    Ok(())
  }
}

