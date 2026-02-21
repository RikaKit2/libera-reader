use std::fmt;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum BookExt {
  PDF,
  EPUB,
  MOBI,
}

impl BookExt {
  // Keep the original name but accept &Path instead of &PathBuf to satisfy clippy::ptr_arg.
  pub fn from_pathbuf(path: &Path) -> Option<Self> {
    match path.extension().and_then(|ext| ext.to_str()).map(|s| s.to_ascii_lowercase()).as_deref() {
      Some("pdf") => Some(Self::PDF),
      Some("epub") => Some(Self::EPUB),
      Some("mobi") => Some(Self::MOBI),
      _ => None,
    }
  }
}

// Implement Display instead of providing an inherent to_string method.
// This also enables the standard `to_string()` via the `ToString` impl.
impl fmt::Display for BookExt {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let s = match self {
      Self::PDF => "pdf",
      Self::EPUB => "epub",
      Self::MOBI => "mobi",
    };
    write!(f, "{}", s)
  }
}
