use crate::db::models::GetBookData;
use crate::db::models::book_data::BookData;
use crate::types::{BookHash, BookPath, BookSize};
use mutool_bindings::mutool_status::MuToolError;
use native_db::*;
#[allow(unused_imports)]
use native_model::{Model, native_model};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[native_model(id = 3, version = 1)]
#[native_db]
pub struct DataOfUnhashedBook {
  #[primary_key]
  pub book_size: BookSize,
  pub book_hash: Option<BookHash>,
  pub book_data: BookData,
}
impl DataOfUnhashedBook {
  pub fn new(book_size: BookSize, books_pk: Vec<BookPath>) -> Self {
    let mut book_data = BookData::new(books_pk);
    match book_size == 0 {
      true => book_data.mutool_err = Some(MuToolError::FileIsEmpty),
      false => {}
    };
    DataOfUnhashedBook { book_size, book_hash: None, book_data }
  }
}
impl GetBookData for DataOfUnhashedBook {
  fn get_book_data(&self) -> &BookData {
    &self.book_data
  }
  fn get_book_data_mut(&mut self) -> &mut BookData {
    &mut self.book_data
  }
}
