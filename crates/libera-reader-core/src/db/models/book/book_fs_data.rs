use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BookFsData {
  pub path_to_dir: String,
  pub dir_name: String,
  pub book_name: String,
  pub ext: String,
}

impl BookFsData {
  pub fn new(path_to_dir: String, dir_name: String, book_name: String, ext: String) -> Self {
    BookFsData { path_to_dir, dir_name, book_name, ext }
  }
  pub(crate) fn from_pathbuf(path: &std::path::PathBuf) -> Self {
    let path = path.to_string_lossy().to_string();
    let path_to_dir = path.clone();
    let dir_name = path.clone();
    let book_name = path.clone();
    let ext = path.clone();
    BookFsData { path_to_dir, dir_name, book_name, ext }
  }
}
