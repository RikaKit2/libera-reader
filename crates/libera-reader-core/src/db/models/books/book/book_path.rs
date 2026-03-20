use std::{
  hash::{Hash, Hasher},
  path::PathBuf,
};

use serde::{Deserialize, Serialize};

use crate::db::models::books::book::{BookDir, BookExt, BookName, BookSize};

#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
pub struct BookPath {
  pub parent_dir: BookDir,
  pub name: BookName,
  pub ext: BookExt,
  pub deleted: bool,
}
impl BookPath {
  pub fn new(path: &std::path::Path) -> Option<Self> {
    match BookExt::from_pathbuf(path) {
      Some(ext) => {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let parent_dir = BookDir::new(path.parent().unwrap().to_path_buf());
        Some(Self { parent_dir, name, ext, deleted: false })
      }
      None => None,
    }
  }
  pub(crate) fn as_pathbuf(&self) -> PathBuf {
    PathBuf::from(&self.parent_dir.inn).join(&self.name).with_extension(self.ext.to_string())
  }
  pub(crate) fn get_book_size(&self) -> anyhow::Result<BookSize> {
    let buf = &self.as_pathbuf();
    BookSize::new(buf)
  }
  pub fn full_path_string(&self) -> String {
    self.as_pathbuf().to_str().unwrap().to_string()
  }
  pub fn exists_on_disk(&self) -> bool {
    self.as_pathbuf().exists()
  }
  pub(crate) fn mark_as_deleted(&mut self) {
    self.deleted = true;
  }
}

impl PartialEq for BookPath {
  fn eq(&self, other: &Self) -> bool {
    self.full_path_string() == other.full_path_string()
  }
}
impl Hash for BookPath {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.full_path_string().hash(state);
  }
}
