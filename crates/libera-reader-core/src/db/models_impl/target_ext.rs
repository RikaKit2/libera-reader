use crate::db::crud;
use crate::db::models_impl::{GetOrCreate, NewModel};
use crate::models::TargetExt;
use native_db::ToInput;


impl TargetExt {
  pub(crate) fn new() -> Self { Self::get_or_create(1) }
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
  pub fn set_pdf(&mut self, value: bool) {
    let mut new_self = self.clone();
    new_self.pdf = value.clone();
    crud::update(self.clone(), new_self).unwrap();
    self.pdf = value;
  }
  pub fn set_epub(&mut self, value: bool) {
    let mut new_self = self.clone();
    new_self.epub = value.clone();
    crud::update(self.clone(), new_self).unwrap();
    self.epub = value;
  }
  pub fn set_mobi(&mut self, value: bool) {
    let mut new_self = self.clone();
    new_self.mobi = value.clone();
    crud::update(self.clone(), new_self).unwrap();
    self.mobi = value;
  }
}
impl Default for TargetExt {
  fn default() -> Self {
    Self::new()
  }
}
impl NewModel for TargetExt {
  fn new_model() -> Self
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
