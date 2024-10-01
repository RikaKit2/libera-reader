use native_db::*;
#[allow(unused_imports)]
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
#[native_model(id = 1, version = 1)]
#[native_db]
pub struct Settings {
  #[primary_key]
  pub id: u64,
  pub language: String,
  pub theme: String,
  pub path_to_scan: Option<String>,
  pub pdf: bool,
  pub epub: bool,
  pub mobi: bool,
  pub number_of_columns: u64,
  pub page_scaling_factor: f64,
  pub thumbnails_scaling_factor: f64,
  pub workers_num: u64,
}

#[derive(Serialize, Deserialize)]
#[native_model(id = 2, version = 1)]
#[native_db]
pub struct BookMark {
  #[primary_key]
  pub id: u64,
  pub title: String,
  pub content: String,
  pub page_number: u64,
  pub book_data_link: String,
  pub time_created: String,
  pub time_updated: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[native_model(id = 3, version = 1)]
#[native_db]
pub struct BookData {
  #[primary_key]
  pub hash: String,
  #[secondary_key]
  pub file_size: String,
  pub cached: bool,
  pub title: Option<String>,
  pub author: Option<String>,
  pub page_count: Option<u64>,
  pub in_history: bool,
  pub favorite: bool,
  pub last_page_number: u64,
  pub latest_opening_in: Option<String>,
}

#[derive(Serialize, Deserialize, Eq, Hash, PartialEq, Clone, Debug)]
#[native_model(id = 4, version = 1)]
#[native_db]
pub struct BookItem {
  #[primary_key]
  pub path_to_book: String,
  pub path_to_dir: String,
  pub dir_name: String,
  pub book_name: String,
  pub ext: String,
  pub book_data_link: Option<String>,
  pub path_is_valid: bool,
}

impl BookItem {
  pub fn from_pathbuf(pathbuf: &PathBuf) -> Self {
    Self {
      path_to_book: pathbuf.to_str().unwrap().to_string(),
      path_to_dir: pathbuf.parent().unwrap().to_str().unwrap().to_string(),
      book_name: pathbuf.file_name().unwrap().to_str().unwrap().to_string(),
      dir_name: pathbuf.parent().unwrap().file_name().unwrap().to_str().unwrap().to_string(),
      ext: pathbuf.extension().unwrap().to_str().unwrap().to_string(),
      book_data_link: Option::from(None),
      path_is_valid: true,
    }
  }
}
