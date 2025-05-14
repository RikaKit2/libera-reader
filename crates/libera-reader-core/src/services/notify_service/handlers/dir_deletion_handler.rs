use crate::app_dirs::AppDirs;
use crate::db::crud;
use crate::services::notify_service::handlers::book_deletion_handler;
use measure_time_macro::measure_time;
use native_db::Database;
use std::sync::{Arc, RwLock};
use tracing::debug;

#[measure_time]
pub(crate) fn dir_deletion_handler(path_to_dir: String, app_dirs: &Arc<RwLock<AppDirs>>, db: &Database) {
  for old_book in crud::book::get_books_located_in_dir(path_to_dir, db) {
    book_deletion_handler(old_book.path_to_book.as_str(), app_dirs, db);
  }
}

