use crate::app_dirs::AppDirs;
use crate::db::crud;
use crate::db::models::Book;
use measure_time_macro::measure_time;
use native_db::Database;
use std::sync::{Arc, RwLock};
use tracing::debug;

#[measure_time]
pub(crate) fn book_deletion_handler(path_to_book: &str, app_dirs: &Arc<RwLock<AppDirs>>, db: &Database) {
  match crud::get_primary::<Book>(path_to_book, db) {
    None => { debug!("book_deletion_handler: book not found: {path_to_book}") }
    Some(old_book) => { crud::book::del_book(old_book, app_dirs, db); }
  };
}

