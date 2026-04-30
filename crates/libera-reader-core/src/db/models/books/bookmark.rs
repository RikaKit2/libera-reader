use gpui::SharedString;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct BookMark {
  pub title: SharedString,
  pub content: SharedString,
  pub page_number: u32,
  pub time_created: SharedString,
  pub time_updated: SharedString,
}
