use crate::db::crud;
use crate::db::models::{Book, TargetExt};
use crate::services::notify_service::handlers::book_adding_handler;
use measure_time_macro::measure_time;
use native_db::Database;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tracing::{debug, error};

#[measure_time]
pub(crate) fn book_path_update_handler(old_path: &PathBuf, new_path: &PathBuf, db: &Database, target_ext: &Arc<RwLock<TargetExt>>) {
  match crud::get_primary::<Book>(old_path.to_str().unwrap(), db) {
    None => {
      error!("book_path_update_handler: book not found: {:?}", old_path);
      book_adding_handler(new_path, db, target_ext);
    }
    Some(book_from_db) => {
      let new_book = Book::from_pathbuf(&new_path, book_from_db.book_data_wrapper_pk.clone());
      crud::update(book_from_db, new_book, db).unwrap();
    }
  }
}

