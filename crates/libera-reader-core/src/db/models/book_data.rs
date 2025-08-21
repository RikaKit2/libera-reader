use crate::types::BookPath;
use mutool_bindings::mutool_status::MuToolError;
#[allow(unused_imports)]
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BookData {
  pub books_pk: Vec<BookPath>,
  pub cached: bool,
  pub mutool_err: Option<MuToolError>,
  pub is_deleted: bool,
  pub favorite: bool,
  pub in_history: bool,
  pub title: Option<String>,
  pub author: Option<String>,
  pub page_count: Option<usize>,
  pub last_page_number: usize,
  pub latest_opening_in: Option<String>,
}

impl BookData {
  pub(crate) fn new(books_pk: Vec<BookPath>) -> Self {
    Self {
      books_pk,
      cached: false,
      mutool_err: None,
      is_deleted: false,
      favorite: false,
      in_history: false,
      title: None,
      author: None,
      page_count: None,
      last_page_number: 0,
      latest_opening_in: None,
    }
  }
}
