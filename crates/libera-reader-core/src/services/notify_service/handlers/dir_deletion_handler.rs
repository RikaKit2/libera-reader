use crate::services::notify_service::handlers::{book_deletion_handler, get_books_located_in_dir};
use tracing::debug;
use measure_time_macro::measure_time;


#[measure_time]
pub(crate) fn dir_deletion_handler(path_to_dir: String) {
  for old_book in get_books_located_in_dir(path_to_dir) {
    book_deletion_handler(old_book.path_to_book.as_str());
  }
}

