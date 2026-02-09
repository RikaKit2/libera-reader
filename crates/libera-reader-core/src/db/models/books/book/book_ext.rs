use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum BookExt {
  PDF,
  EPUB,
  MOBI,
}
impl BookExt {
  pub fn from_pathbuf(path: &PathBuf) -> Option<Self> {
    match path.extension().and_then(|ext| ext.to_str()) {
      Some("pdf") => Some(Self::PDF),
      Some("epub") => Some(Self::EPUB),
      Some("mobi") => Some(Self::MOBI),
      _ => None,
    }
  }
  pub(crate) fn to_string(&self) -> String {
    match self {
      Self::PDF => "pdf".to_string(),
      Self::EPUB => "epub".to_string(),
      Self::MOBI => "mobi".to_string(),
    }
  }
}
