use crate::db::models::book_data::BookData;
use crate::db::models::GetBookData;
use crate::types::{BookHash, BookPath, BookSize};
use native_db::*;
#[allow(unused_imports)]
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[native_model(id = 4, version = 1)]
#[native_db]
pub struct DataOfHashedBook {
  #[secondary_key]
  pub book_size: BookSize,
  #[primary_key]
  pub book_hash: BookHash,
  pub book_data: BookData,
}
impl DataOfHashedBook {
  pub fn new(hash: String, file_size: BookSize, books_pk: Vec<BookPath>) -> Self {
    DataOfHashedBook {
      book_hash: hash,
      book_size: file_size,
      book_data: BookData::new(books_pk),
    }
  }
}
impl GetBookData for DataOfHashedBook {
  fn get_book_data_as_ref(&self) -> &BookData {
    &self.book_data
  }
}
