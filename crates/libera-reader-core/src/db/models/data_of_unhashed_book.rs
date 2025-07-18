use crate::db::crud;
use crate::db::crud::insert;
use crate::db::models::book_data::BookData;
use crate::db::models::data_of_hashed_book::DataOfHashedBook;
use crate::db::models::GetBookData;
use crate::types::{BookHash, BookPath, BookSize};
use native_db::*;
#[allow(unused_imports)]
use native_model::{native_model, Model};
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
  pub fn new(file_size: BookSize, books_pk: Vec<BookPath>) -> Self {
    DataOfUnhashedBook {
      book_size: file_size,
      book_hash: None,
      book_data: BookData::new(books_pk),
    }
  }
  pub(crate) fn replace_to_data_of_hashed_book(self, book_hash: BookHash, db: &Database) {
    let old_book_data = crud::remove::<Self>(self, db).unwrap();
    let new_book_data = DataOfHashedBook {
      book_hash,
      book_size: old_book_data.book_size,
      book_data: old_book_data.book_data,
    };
    insert::<DataOfHashedBook>(new_book_data, db).unwrap();
  }
}
impl GetBookData for DataOfUnhashedBook {
  fn get_book_data_as_ref(&self) -> &BookData {
    &self.book_data
  }
}
