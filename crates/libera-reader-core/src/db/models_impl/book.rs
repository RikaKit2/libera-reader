use crate::db::crud::get_primary;
use crate::models::{Book, BookData, BookDataType, DataOfHashedBook, DataOfUnhashedBook};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;


impl Book {
  pub(crate) fn from_pathbuf(future_book: &PathBuf, book_data_type: BookDataType) -> Self {
    Self {
      path_to_book: future_book.to_str().unwrap().to_string(),
      path_to_dir: future_book.parent().unwrap().to_str().unwrap().to_string(),
      book_name: future_book.file_name().unwrap().to_str().unwrap().to_string(),
      dir_name: future_book.parent().unwrap().file_name().unwrap().to_str().unwrap().to_string(),
      ext: future_book.extension().unwrap().to_str().unwrap().to_string(),
      book_data_pk: book_data_type,
      path_is_valid: true,
    }
  }
  pub(crate) fn get_book_data(&self) -> BookData {
    let book_data = match self.book_data_pk.clone() {
      BookDataType::UniqueSize(book_size) => { get_primary::<DataOfUnhashedBook>(book_size).unwrap().book_data }
      BookDataType::RepeatingSize(book_hash) => { get_primary::<DataOfHashedBook>(book_hash).unwrap().book_data }
    };
    book_data
  }
}
impl Hash for Book {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.path_to_book.hash(state);
  }
}
impl PartialEq for Book {
  fn eq(&self, other: &Self) -> bool {
    self.path_to_book == other.path_to_book
  }
}
impl Eq for Book {}
