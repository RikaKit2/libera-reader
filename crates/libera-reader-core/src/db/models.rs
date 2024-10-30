use crate::db::DB;
use crate::utils::{BookPath, BookSize, Hash};
use gxhash::{HashMap, HashMapExt, HashSet};
use itertools::Itertools;
use native_db::*;
#[allow(unused_imports)]
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
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
  pub file_size: String,
  pub cached: bool,
  pub title: Option<String>,
  pub author: Option<String>,
  pub page_count: Option<u32>,
  pub in_history: bool,
  pub favorite: bool,
  pub last_page_number: u32,
  pub latest_opening_in: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[native_model(id = 4, version = 3)]
#[native_db]
pub struct HashedBookData {
  #[primary_key]
  pub hash: String,
  pub file_size: String,
  pub cached: bool,
  pub title: Option<String>,
  pub author: Option<String>,
  pub page_count: Option<u32>,
  pub in_history: bool,
  pub favorite: bool,
  pub last_page_number: u32,
  pub latest_opening_in: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[native_model(id = 5, version = 3)]
#[native_db]
pub struct BookItem {
  #[primary_key]
  pub path_to_book: String,
  pub path_to_dir: String,
  pub dir_name: String,
  pub book_name: String,
  pub ext: String,
  pub unique_file_size: Option<bool>,
  pub book_data_link: Option<String>,
  pub path_is_valid: bool,
}
pub trait InsertBatch {
  fn insert_batch<T: ToInput>(data: Vec<T>) {
    if data.len() > 0 {
      let rw_conn = DB.rw_transaction().unwrap();
      for i in data {
        rw_conn.insert(i).unwrap();
      }
      rw_conn.commit().unwrap();
    }
  }
}

impl InsertBatch for BookItem {}

impl BookItem {
  pub fn from_pathbuf(pathbuf: &PathBuf) -> Self {
    Self {
      path_to_book: pathbuf.to_str().unwrap().to_string(),
      path_to_dir: pathbuf.parent().unwrap().to_str().unwrap().to_string(),
      book_name: pathbuf.file_name().unwrap().to_str().unwrap().to_string(),
      dir_name: pathbuf.parent().unwrap().file_name().unwrap().to_str().unwrap().to_string(),
      ext: pathbuf.extension().unwrap().to_str().unwrap().to_string(),
      unique_file_size: None,
      book_data_link: Option::from(None),
      path_is_valid: true,
    }
  }

  pub fn get_all() -> HashMap<BookPath, BookItem> {
    let r_conn = DB.r_transaction().unwrap();
    let db_book_items: Vec<BookItem> = r_conn.scan().primary().unwrap().all().unwrap().try_collect().unwrap();
    let mut result: HashMap<BookPath, BookItem> = HashMap::new();
    for i in db_book_items {
      result.insert(i.path_to_book.clone(), i);
    }
    result
  }
}

impl InsertBatch for HashedBookData {}

impl HashedBookData {
  pub fn new(hash: String, file_size: String) -> Self {
    HashedBookData {
      hash,
      file_size,
      cached: false,
      title: None,
      author: None,
      page_count: None,
      in_history: false,
      favorite: false,
      last_page_number: 0,
      latest_opening_in: None,
    }
  }

  pub fn get_all_hashes() -> HashSet<Hash> {
    let r_conn = DB.r_transaction().unwrap();
    let repetitive_book_data: Vec<HashedBookData> = r_conn.scan().primary().unwrap().all().unwrap().try_collect().unwrap();
    let repetitive_book_data: HashSet<Hash> = repetitive_book_data.into_iter().map(|i| i.hash).collect();
    repetitive_book_data
  }
}

impl InsertBatch for BookData {}

impl BookData {
  pub fn new(file_size: String) -> Self {
    BookData {
      file_size,
      cached: false,
      title: None,
      author: None,
      page_count: None,
      in_history: false,
      favorite: false,
      last_page_number: 0,
      latest_opening_in: None,
    }
  }
  pub fn get_all_sizes() -> HashSet<BookSize> {
    let r_conn = DB.r_transaction().unwrap();
    let unique_book_data: Vec<BookData> = r_conn.scan().primary().unwrap().all().unwrap().try_collect().unwrap();
    let unique_book_data: HashSet<BookSize> = unique_book_data.into_iter().map(|i| i.file_size).collect();
    unique_book_data
  }
  pub fn does_size_exist(file_size: String) {
    let r_conn = DB.r_transaction().unwrap();
  }
}

