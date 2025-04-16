use crate::db::models_impl::GetBookData;
use crate::models::{BookData, DataOfHashedBook};
use crate::types::{BookPath, BookSize};


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
