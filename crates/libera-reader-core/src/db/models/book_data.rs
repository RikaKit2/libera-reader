use crate::types::{BookPath, MutoolErr};
#[allow(unused_imports)]
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BookData {
  pub cached: bool,
  pub mutool_err: Option<MutoolErr>,
  pub title: Option<String>,
  pub author: Option<String>,
  pub page_count: Option<i32>,
  pub in_history: bool,
  pub favorite: bool,
  pub last_page_number: i32,
  pub latest_opening_in: Option<String>,
  pub books_pk: Vec<BookPath>,
}

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
