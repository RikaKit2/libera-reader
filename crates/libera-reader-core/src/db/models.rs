use crate::types::{BookHash, BookPath, BookSize, MutoolErr};
use native_db::*;
#[allow(unused_imports)]
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};


#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TextId {
  SetupPageTitle,
  MessageOfSelectingTargetDir,
  SetupPageSelectBtn,
  TargetPath,
  SetupPageNextBtn,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum Lang {
  EN,
  RU
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum ColorScheme {
  Dark,
  Light,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Theme {
  pub name: String,
  pub color_scheme: ColorScheme,
  pub base_100: u32,
  pub base_200: u32,
  pub base_300: u32,
  pub base_color_content: u32,
  pub primary_color: u32,
  pub primary_content_color: u32,
  pub secondary_color: u32,
  pub secondary_content_color: u32,
  pub accent_color: u32,
  pub accent_content_color: u32,
  pub neutral_color: u32,
  pub neutral_content_color: u32,
  pub info_color: u32,
  pub info_content_color: u32,
  pub success_color: u32,
  pub success_content_color: u32,
  pub warning_color: u32,
  pub warning_content_color: u32,
  pub error_color: u32,
  pub error_content_color: u32,
}

#[derive(Serialize, Deserialize, Clone, PartialOrd, PartialEq, Copy, Debug)]
pub enum RootRoute {
  Main(Route),
  BookViewer,
  Setup,
}

#[derive(Serialize, Deserialize, Clone, PartialOrd, PartialEq, Copy, Debug)]
pub enum Route {
  Library,
  FileManager,
  History,
  Favorite,
  BookMarks,
  Stats,
  Settings,
}

#[derive(Serialize, Deserialize, Clone)]
#[native_model(id = 1, version = 1)]
#[native_db]
pub struct Settings {
  #[primary_key]
  pub id: i32,
  pub language: Lang,
  pub theme: Theme,
  pub path_to_scan: Option<String>,
  pub number_of_columns: i32,
  pub page_scaling_factor: f64,
  pub thumbnails_scaling_factor: f64,
  pub workers_num: i32,
  pub route: RootRoute,
  pub setup_is_done: bool,
}

#[derive(Serialize, Deserialize)]
#[native_model(id = 2, version = 1)]
#[native_db]
pub(crate) struct BookMark {
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
pub(crate) struct BookData {
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[native_model(id = 3, version = 1)]
#[native_db]
pub(crate) struct DataOfUnhashedBook {
  #[primary_key]
  pub book_size: BookSize,
  pub book_hash: Option<BookHash>,
  pub book_data: BookData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[native_model(id = 4, version = 1)]
#[native_db]
pub(crate) struct DataOfHashedBook {
  #[secondary_key]
  pub book_size: BookSize,
  #[primary_key]
  pub book_hash: BookHash,
  pub book_data: BookData,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[native_model(id = 5, version = 1)]
#[native_db]
pub struct Book {
  #[primary_key]
  pub path_to_book: String,
  pub path_to_dir: String,
  pub dir_name: String,
  pub book_name: String,
  pub ext: String,
  pub path_is_valid: bool,
  pub book_data_wrapper_pk: BookDataWrapperPK,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum BookDataWrapperPK {
  UniqueSize(BookSize),    // DataOfHashedBook
  RepeatingSize(BookHash), // DataOfUnhashedBook
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[native_model(id = 6, version = 1)]
#[native_db]
pub struct TargetExt {
  #[primary_key]
  pub id: i32,
  pub pdf: bool,
  pub epub: bool,
  pub mobi: bool,
}
