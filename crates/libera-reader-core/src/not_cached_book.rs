use crate::db::crud;
use crate::models::{Book, BookDataType, DataOfHashedBook, DataOfUnhashedBook};
use crate::types::BookPath;
use crate::vars::{APP_DIRS, NOT_CACHED_BOOKS};

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub struct NotCachedBook {
  pub book_path: BookPath,
}

impl NotCachedBook {
  pub(crate) fn new(book_path: BookPath) -> Self {
    Self { book_path }
  }
  pub(crate) fn push_to_storage(self) {
    NOT_CACHED_BOOKS.write().unwrap().push(Box::new(self));
  }
  pub(crate) fn mark_as_cached(self) {
    let book = crud::get_primary::<Book>(self.book_path).unwrap();
    match book.book_data_pk {
      BookDataType::UniqueSize(book_size) => {
        let old_book_data = crud::get_primary::<DataOfUnhashedBook>(book_size).unwrap();
        let mut new_book_data = old_book_data.clone();
        new_book_data.book_data.cached = true;
        crud::update(old_book_data, new_book_data).unwrap();
      }
      BookDataType::RepeatingSize(book_hash) => {
        let old_book_data = crud::get_primary::<DataOfHashedBook>(book_hash).unwrap();
        let mut new_book_data = old_book_data.clone();
        new_book_data.book_data.cached = true;
        crud::update(old_book_data, new_book_data).unwrap();
      }
    }
  }
  pub(crate) fn mark_as_broken(self) {}
  pub(crate) fn get_out_file_name(&self) -> String {
    let book = crud::get_primary::<Book>(self.book_path.clone()).unwrap();
    match &book.book_data_pk {
      BookDataType::UniqueSize(book_size) => APP_DIRS
        .read()
        .unwrap()
        .dir_of_unhashed_books
        .join(book_size)
        .to_str()
        .unwrap()
        .to_string(),
      BookDataType::RepeatingSize(book_hash) => APP_DIRS
        .read()
        .unwrap()
        .dir_of_hashed_books
        .join(book_hash)
        .to_str()
        .unwrap()
        .to_string(),
    }
  }
}
