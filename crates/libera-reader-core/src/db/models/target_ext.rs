use crate::db::crud::update_table;
use crate::db::models::{DefaultModel, GetOrCreate};
use crate::types::DB;
use anyhow::Result;
use native_db::*;
#[allow(unused_imports)]
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[native_model(id = 6, version = 1)]
#[native_db]
pub struct TargetExt {
  #[primary_key]
  pub id: i32,
  pub pdf: bool,
  pub epub: bool,
  pub mobi: bool,
}
impl TargetExt {
  pub fn new(db: &DB) -> Result<Self> { Self::get_or_create(1, db) }
  pub(crate) fn contains(&self, ext: &str) -> bool {
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
impl DefaultModel for TargetExt {
  fn default_model() -> Self
                     where Self: Sized + ToInput, {
    Self {
      id: 1,
      pdf: true,
      epub: false,
      mobi: false,
    }
  }
}
impl GetOrCreate for TargetExt {}
