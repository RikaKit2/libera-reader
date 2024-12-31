use crate::db::crud;
use crate::db::models::Book;
use measure_time_macro::measure_time;
use tracing::debug;


#[measure_time]
pub(crate) fn book_deletion_handler(path_to_book: &str) {
  match crud::get_primary::<Book>(path_to_book) {
    None => {
      debug!("book_deletion_handler: book not found: {path_to_book}")
    }
    Some(old_book) => {
      crud::book::del_book_and_its_data(old_book);
    }
  };
}
