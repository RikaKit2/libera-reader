mod del_book;
mod add_book;

use crate::db::crud;
use crate::db::models::{Book, BookDataWrapperPK, DataOfHashedBook, DataOfHashedBookKey, DataOfUnhashedBook};
use crate::types::{BookPath, BookSize};
pub use add_book::add_book;
pub use del_book::del_book;
use itertools::Itertools;
use native_db::Database;
use std::path::PathBuf;


pub(crate) fn get_num_of_books_of_this_size(book_size: BookSize, db: &Database) -> (usize, Option<DataOfUnhashedBook>) {
  let mut out_data: Option<DataOfUnhashedBook> = None;
  let mut num_of_book_with_this_size = 0;
  match crud::get_primary::<DataOfUnhashedBook>(book_size.clone(), db) {
    None => {
      let r_conn = db.r_transaction().unwrap();
      for i in r_conn.scan().secondary::<DataOfHashedBook>(DataOfHashedBookKey::book_size).unwrap().all().unwrap() {
        match i {
          Ok(_data) => { num_of_book_with_this_size += 1; }
          Err(_) => {}
        }
      }
    }
    Some(data) => {
      num_of_book_with_this_size = data.book_data.books_pk.len();
      out_data = Some(data);
    }
  };
  (num_of_book_with_this_size, out_data)
}
pub(crate) fn update_book_data_type(book_path: BookPath, book_data_type: BookDataWrapperPK, db: &Database) {
  let old_book = crud::get_primary::<Book>(book_path, db).unwrap();
  let mut new_book = old_book.clone();
  new_book.book_data_wrapper_pk = book_data_type;
  crud::update(old_book, new_book, db).unwrap();
}
pub(crate) fn get_books_located_in_dir(path_to_dir: String, db: &Database) -> Vec<Book> {
  let r_conn = db.r_transaction().unwrap();
  let books: Vec<Book> = r_conn.scan().primary().unwrap().start_with(path_to_dir).unwrap().try_collect().unwrap();
  books
}
pub(crate) fn update_the_books_directory(old_dir_path: &PathBuf, new_dir_path: &PathBuf, db: &Database) {
  for old_book in get_books_located_in_dir(old_dir_path.to_str().unwrap().to_string(), db) {
    let mut new_book = old_book.clone();
    new_book.dir_name = new_dir_path.file_name().unwrap().to_str().unwrap().to_string();
    new_book.path_to_dir = new_dir_path.to_str().unwrap().to_string();
    new_book.path_to_book = new_dir_path.join(&old_book.book_name).to_str().unwrap().to_string();
    crud::update(old_book, new_book, db).unwrap();
  }
}
