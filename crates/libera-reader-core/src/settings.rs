use crate::db::crud::update_table;
use crate::db::models::settings::Settings;
use crate::db::models::{GetOrCreate, Lang, RootRoute};
use crate::types::DB;
use anyhow::Result;
use std::ops::{Deref, DerefMut};

pub struct SETTINGS {
  inn: Settings,
  db: DB
}
impl SETTINGS {
  pub fn new(db: DB) -> Result<Self> { Ok(Self { inn: Settings::get_or_create(1, &db)?, db }) }
  pub fn set_path_to_scan(&mut self, new_path: String) -> Result<()> {
    let flag = match &self.path_to_scan {
      None => { true }
      Some(old_path) => { old_path.eq(&new_path) }
    };
    match flag {
      true => {
        update_table(&self.db, Some(self.clone()), |model|
          model.path_to_scan = Some(new_path.clone()))?;
        self.path_to_scan = Some(new_path);
      }
      false => {}
    }
    Ok(())
  }
  pub fn set_language(&mut self, lang: Lang) -> Result<()> {
    match &self.language.eq(&lang) {
      true => {}
      false => {
        update_table(&self.db, Some(self.clone()), |model|
          model.language = lang.clone())?;
        self.language = lang;
      }
    };
    Ok(())
  }
  pub fn set_route(&mut self, new_route: RootRoute) -> Result<()> {
    match &self.route.eq(&new_route) {
      true => {}
      false => {
        update_table(&self.db, Some(self.clone()), |model|
          model.route = new_route.clone())?;
        self.route = new_route;
      }
    }
    Ok(())
  }
  pub fn set_setup_status(&mut self, status: bool) -> Result<()> {
    match &self.setup_is_done.eq(&status) {
      true => {}
      false => {
        update_table(&self.db, Some(self.clone()), |model|
          model.setup_is_done = status)?;
        self.setup_is_done = status;
      }
    }
    Ok(())
  }
  pub fn contains_ext(&self, ext: &str) -> bool {
    let ext_is_pdf = ext.eq("pdf") && self.pdf;
    let ext_is_epub = ext.eq("epub") && self.epub;
    let ext_is_mobi = ext.eq("mobi") && self.mobi;
    if ext_is_pdf || ext_is_epub || ext_is_mobi {
      true
    } else {
      false
    }
  }
  pub fn invert_pdf(&mut self, db: &DB) -> Result<()> {
    update_table(db, Some(self.clone()), |target_ext| target_ext.pdf = !target_ext.pdf)?;
    self.pdf = !self.pdf;
    Ok(())
  }
  pub fn invert_epub(&mut self, db: &DB) -> Result<()> {
    update_table(db, Some(self.clone()), |target_ext| target_ext.epub = !target_ext.epub)?;
    self.epub = !self.epub;
    Ok(())
  }
  pub fn invert_mobi(&mut self, db: &DB) -> Result<()> {
    update_table(db, Some(self.clone()), |target_ext| target_ext.mobi = !target_ext.mobi)?;
    self.mobi = !self.mobi;
    Ok(())
  }
}
impl Deref for SETTINGS {
  type Target = Settings;

  fn deref(&self) -> &Self::Target {
    &self.inn
  }
}
impl DerefMut for SETTINGS {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.inn
  }
}
