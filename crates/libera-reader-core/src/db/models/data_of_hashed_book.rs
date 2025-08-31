use crate::db::DB;
use crate::db::models::GetBookData;
use crate::db::models::book_data::BookData;
use crate::types::{BookHash, BookPath, BookSize};
use mutool_bindings::mutool_status::MuToolError;
use native_db::*;
#[allow(unused_imports)]
use native_model::{Model, native_model};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[native_model(id = 4, version = 1)]
#[native_db]
pub struct DataOfHashedBook {
  #[primary_key]
  pub book_hash: BookHash,
  #[secondary_key]
  pub book_size: BookSize,
  pub book_data: BookData,
}
impl DataOfHashedBook {
  pub fn new(book_hash: BookHash, book_size: BookSize, books_pk: Vec<BookPath>) -> Self {
    let mut book_data = BookData::new(books_pk);
    match book_size == 0 {
      true => book_data.mutool_err = Some(MuToolError::FileIsEmpty),
      false => {}
    };
    DataOfHashedBook { book_hash, book_size, book_data }
  }
  pub(crate) fn new_with_book_data(book_hash: BookHash, book_size: BookSize, book_data: BookData) -> Self {
    DataOfHashedBook { book_hash, book_size, book_data }
  }
  pub(crate) fn scan_by_size(book_size: BookSize, db: &DB) -> anyhow::Result<Vec<Self>> {
    db.scan_secondary_by::<BookSize, Self>(book_size, DataOfHashedBookKey::book_size)
  }
}

impl GetBookData for DataOfHashedBook {
  fn get_book_data(&self) -> &BookData {
    &self.book_data
  }
  fn get_book_data_mut(&mut self) -> &mut BookData {
    &mut self.book_data
  }
}
