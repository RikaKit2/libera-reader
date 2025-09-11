use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct BookMark {
  pub title: String,
  pub content: String,
  pub page_number: u32,
  pub book_data_link: String,
  pub time_created: String,
  pub time_updated: String,
}
