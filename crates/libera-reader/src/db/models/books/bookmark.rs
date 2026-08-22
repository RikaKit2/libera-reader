use gpui::SharedString;
use native_db::*;
use native_model::{Model, native_model};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct BookMark {
  pub title: SharedString,
  pub content: SharedString,
  pub page_number: u32,
  pub time_created: SharedString,
  pub time_updated: SharedString,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[native_model(id = 5, version = 1)]
#[native_db]
pub struct BookBookmarks {
  #[primary_key]
  pub book_id: String,
  pub items: Vec<BookMark>,
}

impl BookBookmarks {
  pub fn new(book_id: String, items: Vec<BookMark>) -> Self {
    Self { book_id, items }
  }
}
