use std::fmt;
use std::path::Path;

use gpui::SharedString;
use serde::{Deserialize, Serialize};

pub type RealExtStr = SharedString;
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum BookExt {
  PDF(RealExtStr),
  EPUB(RealExtStr),
  MOBI(RealExtStr),
}

impl BookExt {
  pub fn from_pathbuf(path: &Path) -> Option<Self> {
    match path.extension() {
      Some(ext) => {
        let real_ext = ext.to_str().unwrap().to_string();
        let ext_lowercase = real_ext.to_ascii_lowercase();
        let real_ext: RealExtStr = real_ext.into();
        match ext_lowercase.as_str() {
          "pdf" => Some(Self::PDF(real_ext)),
          "epub" => Some(Self::EPUB(real_ext)),
          "mobi" => Some(Self::MOBI(real_ext)),
          _ => None,
        }
      }
      None => None,
    }
  }
}

impl fmt::Display for BookExt {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let s = match self {
      Self::PDF(ext) => ext,
      Self::EPUB(ext) => ext,
      Self::MOBI(ext) => ext,
    };
    write!(f, "{}", s)
  }
}
