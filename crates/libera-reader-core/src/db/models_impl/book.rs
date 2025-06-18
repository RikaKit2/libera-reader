use crate::db::crud::get_primary;
use crate::db::models::{Book, BookData, BookDataWrapperPK, DataOfHashedBook, DataOfUnhashedBook};
use crate::types::{APP_DIRS, BookPath, MutoolErr};
use itertools::Itertools;
use native_db::Database;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;


impl Book {
  pub(crate) fn from_pathbuf(future_book: &PathBuf, book_data_type: BookDataWrapperPK) -> Self {
    Self {
      path_to_book: future_book.to_str().unwrap().to_string(),
      path_to_dir: future_book.parent().unwrap().to_str().unwrap().to_string(),
      book_name: future_book.file_name().unwrap().to_str().unwrap().to_string(),
      dir_name: future_book.parent().unwrap().file_name().unwrap().to_str().unwrap().to_string(),
      ext: future_book.extension().unwrap().to_str().unwrap().to_string(),
      book_data_wrapper_pk: book_data_type,
      path_is_valid: true,
    }
  }
  pub(crate) fn get_book_data(&self, db: &Database) -> BookData {
    let book_data = match self.book_data_wrapper_pk.clone() {
      BookDataWrapperPK::UniqueSize(book_size) => get_primary::<DataOfUnhashedBook>(book_size, db).unwrap().book_data,
      BookDataWrapperPK::RepeatingSize(book_hash) => get_primary::<DataOfHashedBook>(book_hash, db).unwrap().book_data,
    };
    book_data
  }
  pub(crate) fn mark_as_cached(self, db: &Database) {
    self.book_data_wrapper_pk.update_book_data(|book_data: &mut BookData| {
      book_data.cached = true;
    }, db)
  }
  pub(crate) fn mark_as_broken(self, mutool_err: MutoolErr, db: &Database) {
    self.book_data_wrapper_pk.update_book_data(|book_data: &mut BookData| {
      book_data.mutool_err = Some(mutool_err);
      book_data.cached = false;
    }, db)
  }
  pub(crate) fn get_path_to_storage(&self, app_dirs: &APP_DIRS) -> String {
    match &self.book_data_wrapper_pk {
      BookDataWrapperPK::UniqueSize(book_size) =>
        app_dirs.read().unwrap().inn.dir_of_unhashed_books.join(book_size).to_str().unwrap().to_string(),
      BookDataWrapperPK::RepeatingSize(book_hash) =>
        app_dirs.read().unwrap().inn.dir_of_hashed_books.join(book_hash).to_str().unwrap().to_string()
    }
  }
  pub fn get_by_path(path_to_book: &BookPath, db: &Database) -> Option<Book> {
    get_primary::<Book>(path_to_book.clone(), db)
  }
  pub fn get_all(db: &Database) -> Vec<Book> {
    let r_conn = db.r_transaction().unwrap();
    r_conn.scan().primary().unwrap().all().unwrap().try_collect().unwrap()
  }
}
impl Hash for Book {
  fn hash<H: Hasher>(&self, state: &mut H) { self.path_to_book.hash(state); }
}
impl PartialEq for Book {
  fn eq(&self, other: &Self) -> bool { self.path_to_book == other.path_to_book }
}
impl Eq for Book {}
