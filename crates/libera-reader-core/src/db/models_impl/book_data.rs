use crate::db::models::BookData;
use crate::types::BookPath;


impl BookData {
  pub(crate) fn new(books_pk: Vec<BookPath>) -> Self {
    Self {
      cached: false,
      mutool_err: None,
      title: None,
      author: None,
      page_count: None,
      in_history: false,
      favorite: false,
      last_page_number: 0,
      latest_opening_in: None,
      books_pk,
    }
  }
}
