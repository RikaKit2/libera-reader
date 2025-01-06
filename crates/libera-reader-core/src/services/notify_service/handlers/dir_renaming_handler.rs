use crate::db::crud;
use crate::services::notify_service::handlers::get_books_located_in_dir;
use measure_time_macro::measure_time;
use std::path::PathBuf;
use tracing::debug;


#[measure_time]
pub(crate) fn dir_renaming_handler(old_dir_path: String, new_dir_path: &PathBuf) {
  for old_book in get_books_located_in_dir(old_dir_path) {
    let mut new_book = old_book.clone();
    new_book.dir_name = new_dir_path.file_name().unwrap().to_str().unwrap().to_string();
    new_book.path_to_dir = new_dir_path.to_str().unwrap().to_string();
    new_book.path_to_book = new_dir_path.join(&old_book.book_name).to_str().unwrap().to_string();
    crud::update(old_book, new_book).unwrap();
  }
}
