use crate::db::models::book_data::BookData;
use crate::db::models::GetBookData;
use crate::types::{BookHash, BookPath, BookSize, DB};
use anyhow::Result;
use itertools::Itertools;
use native_db::*;
#[allow(unused_imports)]
use native_model::{native_model, Model};
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
    DataOfHashedBook {
      book_hash,
      book_size,
      book_data: BookData::new(books_pk),
    }
  }
  pub(crate) fn new_with_other_book_data(book_hash: BookHash, book_size: BookSize, book_data: BookData) -> Self {
    DataOfHashedBook {
      book_hash,
      book_size,
      book_data
    }
  }
  pub(crate) fn find_by_size(book_size: BookSize, db: &DB) -> Result<Vec<Self>> {
    let res: Vec<DataOfHashedBook> = db.r_transaction()?
      .scan()
      .secondary(DataOfHashedBookKey::book_size)?
      .start_with(book_size.clone())?
      .try_collect()?;
    Ok(res)
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
