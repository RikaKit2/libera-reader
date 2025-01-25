use crate::db::crud;
use crate::models::Book;
use crate::utils::calc_file_size_in_mb;
use crate::vars::TARGET_EXT;
use measure_time_macro::measure_time;
use std::path::PathBuf;
use std::time::Duration;
use tracing::{debug, error};


#[measure_time]
pub(crate) fn book_adding_handler(bookbuf: &PathBuf) -> Duration {
  let start_time = std::time::Instant::now();
  let ext = bookbuf.extension().unwrap().to_str().unwrap().to_string();
  if TARGET_EXT.read().unwrap().contains(&ext) {
    let book_size = calc_file_size_in_mb(bookbuf);
    crud::book::add_book(bookbuf, book_size);
  }
  let total_time = start_time.elapsed();
  debug!("Function book_adding_handler executed in: {:?}", &total_time);
  total_time
}

#[measure_time]
pub(crate) fn book_deletion_handler(path_to_book: &str) {
  match crud::get_primary::<Book>(path_to_book) {
    None => { debug!("book_deletion_handler: book not found: {path_to_book}") }
    Some(old_book) => { crud::book::del_book_and_its_data(old_book); }
  };
}

#[measure_time]
pub(crate) fn book_path_update_handler(old_path: &PathBuf, new_path: &PathBuf) {
  match crud::get_primary::<Book>(old_path.to_str().unwrap()) {
    None => {
      error!("book_path_update_handler: book not found: {:?}", old_path);
      book_adding_handler(new_path);
    }
    Some(book_from_db) => {
      let new_book = Book::from_pathbuf(&new_path, book_from_db.book_data_pk.clone());
      crud::update(book_from_db, new_book).unwrap();
    }
  }
}

#[measure_time]
pub(crate) fn dir_deletion_handler(path_to_dir: String) {
  for old_book in crud::book::get_books_located_in_dir(path_to_dir) {
    book_deletion_handler(old_book.path_to_book.as_str());
  }
}

