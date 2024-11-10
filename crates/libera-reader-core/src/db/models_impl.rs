use crate::db::models::{AppLanguage, AppTheme, Book, BookData, RepeatSizeBookData, Settings, UniqueSizeBookData};
use crate::db::DB;
use crate::utils::{BookPath, BookSize};
use itertools::Itertools;
use std::path::PathBuf;


impl Default for Settings {
  fn default() -> Self {
    Self {
      id: 1,
      language: AppLanguage::EN,
      theme: AppTheme::Sunset,
      path_to_scan: None,
      pdf: true,
      epub: false,
      mobi: false,
      number_of_columns: 6,
      page_scaling_factor: 1.0,
      thumbnails_scaling_factor: 4.0,
      workers_num: 2,
    }
  }
}

impl Settings {
  pub fn new() -> Self { Self::default() }
}
impl Book {
  pub fn from_pathbuf(book_path: &PathBuf) -> Self {
    Self {
      path_to_book: book_path.to_str().unwrap().to_string(),
      path_to_dir: book_path.parent().unwrap().to_str().unwrap().to_string(),
      book_name: book_path.file_name().unwrap().to_str().unwrap().to_string(),
      dir_name: book_path.parent().unwrap().file_name().unwrap().to_str().unwrap().to_string(),
      ext: book_path.extension().unwrap().to_str().unwrap().to_string(),
      book_data_primary_key: None,
      path_is_valid: true,
    }
  }
  pub fn get_all_from_db() -> Vec<Book> {
    let r_conn = DB.r_transaction().unwrap();
    r_conn.scan().primary().unwrap().all().unwrap().try_collect().unwrap()
  }
}

impl RepeatSizeBookData {
  pub fn new(hash: String, file_size: BookSize, books_pk: Vec<BookPath>) -> Self {
    RepeatSizeBookData {
      book_hash: hash,
      file_size,
      book_data: BookData {
        cached: false,
        title: None,
        author: None,
        page_count: None,
        in_history: false,
        favorite: false,
        last_page_number: 0,
        latest_opening_in: None,
        books_pk,
      },
    }
  }
}
impl UniqueSizeBookData {
  pub fn new(file_size: BookSize, books_pk: Vec<BookPath>) -> Self {
    UniqueSizeBookData {
      file_size,
      book_hash: None,
      book_data: BookData {
        cached: false,
        title: None,
        author: None,
        page_count: None,
        in_history: false,
        favorite: false,
        last_page_number: 0,
        latest_opening_in: None,
        books_pk,
      },
    }
  }
  pub fn to_repeat_size_book_data(self, book_hash: String) -> RepeatSizeBookData {
    RepeatSizeBookData {
      book_hash,
      file_size: self.file_size,
      book_data: self.book_data,
    }
  }
}

pub trait GetBookData {
  fn get_book_data(self) -> BookData;
  fn get_book_data_as_ref(&self) -> &BookData;
}
impl GetBookData for UniqueSizeBookData {
  fn get_book_data(self) -> BookData {
    self.book_data
  }
  fn get_book_data_as_ref(&self) -> &BookData {
    &self.book_data
  }
}
impl GetBookData for RepeatSizeBookData {
  fn get_book_data(self) -> BookData {
    self.book_data
  }
  fn get_book_data_as_ref(&self) -> &BookData {
    &self.book_data
  }
}
