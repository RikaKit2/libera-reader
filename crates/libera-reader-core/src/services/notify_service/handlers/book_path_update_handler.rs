use crate::db::crud;
use crate::models::Book;
use crate::services::notify_service::handlers;
use measure_time_macro::measure_time;
use std::path::PathBuf;
use tracing::{debug, error};


#[measure_time]
pub(crate) fn book_path_update_handler(old_path: &PathBuf, new_path: &PathBuf) {
  match crud::get_primary::<Book>(old_path.to_str().unwrap()) {
    None => {
      error!("book_path_update_handler: book not found: {:?}", old_path);
      handlers::book_adding_handler(new_path);
    }
    Some(book_from_db) => {
      let new_book = Book::from_pathbuf(&new_path, book_from_db.book_data_pk.clone());
      crud::update(book_from_db, new_book).unwrap();
    }
  }
}
