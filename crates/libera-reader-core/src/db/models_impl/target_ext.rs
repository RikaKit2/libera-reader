use crate::db::crud;
use crate::db::models::TargetExt;
use crate::db::models_impl::{DefaultModel, GetOrCreate};
use native_db::ToInput;
use crate::types::DB;


impl TargetExt {
  pub fn new(db: &DB) -> Self { Self::get_or_create(1, db) }
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
  pub fn set_pdf(&mut self, value: bool, db: &DB) {
    let mut new_self = self.clone();
    new_self.pdf = value.clone();
    crud::update(self.clone(), new_self, db).unwrap();
    self.pdf = value;
  }
  pub fn set_epub(&mut self, value: bool, db: &DB) {
    let mut new_self = self.clone();
    new_self.epub = value.clone();
    crud::update(self.clone(), new_self, db).unwrap();
    self.epub = value;
  }
  pub fn set_mobi(&mut self, value: bool, db: &DB) {
    let mut new_self = self.clone();
    new_self.mobi = value.clone();
    crud::update(self.clone(), new_self, db).unwrap();
    self.mobi = value;
  }
}
impl DefaultModel for TargetExt {
  fn default_model() -> Self
  where
    Self: Sized + ToInput,
  {
    Self {
      id: 1,
      pdf: true,
      epub: false,
      mobi: false,
    }
  }
}
impl GetOrCreate for TargetExt {}
