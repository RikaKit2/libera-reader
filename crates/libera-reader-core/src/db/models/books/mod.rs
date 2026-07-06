pub mod book;
pub(crate) mod book_hashes;
pub(crate) mod book_sizes;
pub mod bookmark;
pub(crate) mod mutool_data;
pub(crate) mod user_data;
use gpui::SharedString;
use std::hash::Hash;

use crate::db::models::MutoolData;

use native_db::{Key, ToKey};
#[allow(unused_imports)]
use native_model::{Model, native_model};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct BookHash(pub SharedString);

impl From<SharedString> for BookHash {
  fn from(value: SharedString) -> Self {
    Self(value)
  }
}

impl From<String> for BookHash {
  fn from(value: String) -> Self {
    Self(value.into())
  }
}

impl ToKey for BookHash {
  fn to_key(&self) -> Key {
    Key::new(self.0.as_bytes().to_vec())
  }

  fn key_names() -> Vec<String> {
    vec!["BookHash".into()]
  }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum DuplicateBookData {
  BookHash(BookHash),
  MutoolData(Option<MutoolData>),
}

#[derive(Serialize, Deserialize, Clone)]
pub enum BookType {
  UniqueSize { book_path: book::BookPath, mutool_data: Option<MutoolData> },
  DuplicateSize(crate::types::HashMap<book::BookPath, DuplicateBookData>),
}
