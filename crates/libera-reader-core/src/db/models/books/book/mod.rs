use std::{
  hash::{Hash, Hasher},
  path::PathBuf,
};

use gpui::SharedString;
use serde::{Deserialize, Serialize};

use crate::db::models::{BookMark, UserData};

pub(crate) type BookName = SharedString;
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
  #[serde(default)]
  pub bookmarks: Vec<BookMark>,
}

impl Book {
  pub(crate) fn new(book_path: BookPath) -> anyhow::Result<Self> {
    let book_size = book_path.get_book_size().unwrap();
    Ok(Self { book_path, book_size, user_data: UserData::default(), bookmarks: Vec::new() })
  }

  pub fn exists_on_disk(&self) -> bool {
    self.book_path.exists_on_disk()
  }
  pub(crate) fn pathbuf(&self) -> PathBuf {
    self.book_path.as_pathbuf()
  }
  pub(crate) fn full_path_str(&self) -> SharedString {
    self.pathbuf().to_str().unwrap().to_string().into()
  }
  pub fn can_delete(&self) -> bool {
    !self.user_data.favorite && self.user_data.last_opened == 0 && self.bookmarks.is_empty()
  }

  pub fn add_bookmark(&mut self, bookmark: BookMark) {
    self.bookmarks.push(bookmark);
  }

  pub fn update_bookmark(&mut self, bookmark: BookMark) -> bool {
    if let Some(existing) =
      self.bookmarks.iter_mut().find(|b| b.time_created == bookmark.time_created)
    {
      *existing = bookmark;
      true
    } else {
      false
    }
  }

  pub fn remove_bookmark(&mut self, time_created: &str) -> bool {
    let old_len = self.bookmarks.len();
    self.bookmarks.retain(|b| b.time_created.as_ref() != time_created);
    old_len != self.bookmarks.len()
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
