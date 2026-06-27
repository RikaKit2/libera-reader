use std::{
  hash::{Hash, Hasher},
  path::{Path, PathBuf},
};

use gpui::SharedString;
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
  pub fn new(path: &Path) -> Option<Self> {
    match BookExt::from_pathbuf(path) {
      Some(ext) => {
        let name = path.file_stem()?.to_string_lossy().to_string().into();
        let parent_dir = BookDir::new(path.parent()?.to_path_buf());
        Some(Self { parent_dir, name, ext, deleted: false })
      }
      None => None,
    }
  }

  pub fn file_name(&self) -> BookName {
    let name = self.name.as_ref();
    let ext = self.ext.to_string();
    let expected_suffix = format!(".{}", ext);

    if name.to_ascii_lowercase().ends_with(&expected_suffix.to_ascii_lowercase()) {
      self.name.clone()
    } else {
      format!("{}.{}", name, ext).into()
    }
  }

  pub fn display_name(&self) -> BookName {
    let name = self.name.as_ref();
    let ext = self.ext.to_string();
    let expected_suffix = format!(".{}", ext);

    if name.to_ascii_lowercase().ends_with(&expected_suffix.to_ascii_lowercase()) {
      name[..name.len() - expected_suffix.len()].to_string().into()
    } else {
      self.name.clone()
    }
  }

  pub(crate) fn as_pathbuf(&self) -> PathBuf {
    let file_name = self.file_name();
    PathBuf::from(&self.parent_dir.inn).join(file_name.as_ref())
  }

  pub(crate) fn get_book_size(&self) -> anyhow::Result<BookSize> {
    let buf = &self.as_pathbuf();
    BookSize::new(buf)
  }
  pub fn full_path_string(&self) -> SharedString {
    self.as_pathbuf().to_str().unwrap().to_string().into()
  }
  /// Reconstruct a BookPath from an id (full path string)
  pub fn from_id(id: &str) -> Self {
    let path = Path::new(id);
    Self::new(path).unwrap_or_else(|| {
      // Fallback: create a minimal BookPath
      let parent_dir = BookDir::new(path.parent().map(|p| p.to_path_buf()).unwrap_or_default());
      let name = path.file_stem().unwrap_or_default().to_string_lossy().to_string().into();
      let ext = BookExt::from_pathbuf(path).unwrap_or(BookExt::PDF("pdf".into()));
      Self { parent_dir, name, ext, deleted: false }
    })
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
