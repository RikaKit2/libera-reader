use crate::db::crud;
use crate::db::crud::insert;
use crate::db::models_impl::GetBookData;
use crate::models::{BookData, DataOfHashedBook, DataOfUnhashedBook};
use crate::types::{BookHash, BookPath, BookSize};


impl DataOfUnhashedBook {
  pub fn new(file_size: BookSize, books_pk: Vec<BookPath>) -> Self {
    DataOfUnhashedBook {
      book_size: file_size,
      book_hash: None,
      book_data: BookData::new(books_pk),
    }
  }
  pub(crate) fn replace_to_data_of_hashed_book(self, book_hash: BookHash) {
    let old_book_data = crud::remove::<Self>(self).unwrap();
    let new_book_data = DataOfHashedBook {
      book_hash,
      book_size: old_book_data.book_size,
      book_data: old_book_data.book_data,
    };
    insert::<DataOfHashedBook>(new_book_data).unwrap();
  }
}
impl GetBookData for DataOfUnhashedBook {
  fn get_book_data_as_ref(&self) -> &BookData {
    &self.book_data
  }
}
