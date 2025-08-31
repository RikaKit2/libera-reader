use native_db::*;
#[allow(unused_imports)]
use native_model::{Model, native_model};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[native_model(id = 2, version = 1)]
#[native_db]
pub struct BookMark {
  #[primary_key]
  pub id: u32,
  pub title: String,
  pub content: String,
  pub page_number: u32,
  pub book_data_link: String,
  pub time_created: String,
  pub time_updated: String,
}
