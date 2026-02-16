use std::{
  hash::{Hash, Hasher},
  path::PathBuf,
};

use serde::{Deserialize, Serialize};

use crate::db::models::UserData;
pub(crate) type BookName = String;
mod book_dir;
mod book_ext;
mod book_path;
mod book_size;

pub use book_dir::BookDir;
pub use book_ext::BookExt;
pub use book_path::BookPath;
pub use book_size::BookSize;

#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
pub struct Book {
  pub book_path: BookPath,
  pub book_size: BookSize,
  pub user_data: UserData,
}
impl Book {
  pub(crate) async fn new(book_path: BookPath) -> anyhow::Result<Self> {
    let book_size = book_path.get_book_size().await.unwrap();
    Ok(Self { book_path, book_size, user_data: UserData::default() })
  }
  pub fn exists_on_disk(&self) -> bool {
    self.book_path.exists_on_disk()
  }
  pub(crate) fn pathbuf(&self) -> PathBuf {
    self.book_path.as_pathbuf()
  }
  pub(crate) fn full_path_str(&self) -> String {
    self.pathbuf().to_str().unwrap().to_string()
  }
  pub fn can_delete(&self) -> bool {
    self.user_data.favorite == false && self.user_data.in_history == false
  }
  pub(crate) fn mark_as_deleted(&mut self) {
    self.book_path.mark_as_deleted();
  }
}
impl PartialEq for Book {
  fn eq(&self, other: &Self) -> bool {
    self.full_path_str() == other.full_path_str()
  }
}
impl Hash for Book {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.full_path_str().hash(state);
  }
}
