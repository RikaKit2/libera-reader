use std::{
  hash::{Hash, Hasher},
  path::PathBuf,
};

use native_db::{Key, ToKey};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Eq, Clone, Debug)]
pub struct BookDir {
  pub(crate) inn: PathBuf,
}
impl BookDir {
  pub(crate) fn new(dir_path: PathBuf) -> Self {
    Self { inn: dir_path }
  }
  pub fn dir_name(&self) -> String {
    self.inn.file_name().unwrap().to_string_lossy().to_string()
  }
  pub fn full_path(&self) -> String {
    self.inn.to_string_lossy().to_string()
  }
  pub fn exists(&self) -> bool {
    self.inn.exists()
  }
}
impl PartialEq for BookDir {
  fn eq(&self, other: &Self) -> bool {
    self.inn == other.inn
  }
}
impl Hash for BookDir {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.inn.hash(state);
  }
}

impl ToKey for BookDir {
  fn to_key(&self) -> Key {
    Key::new(self.inn.to_string_lossy().to_string().as_bytes().to_vec())
  }

  fn key_names() -> Vec<String> {
    vec!["BookDir".to_string()]
  }
}
