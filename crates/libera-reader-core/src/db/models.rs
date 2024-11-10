use native_db::*;
#[allow(unused_imports)]
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use crate::utils::{BookPath, BookSize, BookHash};


#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) enum BookDataType {
  UniqueSizeBook(BookSize),
  RepeatingSizeBook(Option<BookHash>),
}

#[derive(Serialize, Deserialize, Clone)]
pub enum AppLanguage {
  EN
}
#[derive(Serialize, Deserialize, Clone)]
pub enum AppTheme {
  Sunset,
  Dark
}

#[derive(Serialize, Deserialize, Clone)]
#[native_model(id = 1, version = 1)]
#[native_db]
pub struct Settings {
  #[primary_key]
  pub id: i32,
  pub language: AppLanguage,
  pub theme: AppTheme,
  pub path_to_scan: Option<String>,
  pub pdf: bool,
  pub epub: bool,
  pub mobi: bool,
  pub number_of_columns: i32,
  pub page_scaling_factor: f64,
  pub thumbnails_scaling_factor: f64,
  pub workers_num: i32,
}

#[derive(Serialize, Deserialize)]
#[native_model(id = 2, version = 1)]
#[native_db]
pub struct BookMark {
  #[primary_key]
  pub id: i32,
  pub title: String,
  pub content: String,
  pub page_number: i32,
  pub book_data_link: String,
  pub time_created: String,
  pub time_updated: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BookData {
  pub cached: bool,
  pub title: Option<String>,
  pub author: Option<String>,
  pub page_count: Option<i32>,
  pub in_history: bool,
  pub favorite: bool,
  pub last_page_number: i32,
  pub latest_opening_in: Option<String>,
  pub books_pk: Vec<BookPath>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[native_model(id = 3, version = 1)]
#[native_db]
pub struct UniqueSizeBookData {
  pub book_hash: Option<String>,
  #[primary_key]
  pub file_size: BookSize,
  pub book_data: BookData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[native_model(id = 4, version = 3)]
#[native_db]
pub struct RepeatSizeBookData {
  #[primary_key]
  pub book_hash: BookHash,
  #[secondary_key]
  pub file_size: String,
  pub book_data: BookData,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[native_model(id = 5, version = 3)]
#[native_db]
pub struct Book {
  #[primary_key]
  pub path_to_book: String,
  pub path_to_dir: String,
  pub dir_name: String,
  pub book_name: String,
  pub ext: String,
  pub path_is_valid: bool,
  pub book_data_primary_key: Option<BookDataType>,
}
