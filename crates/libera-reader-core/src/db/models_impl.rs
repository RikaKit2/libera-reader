use crate::db::models::{Book, BookData, DataOfHashedBook, DataOfUnhashedBook,
                        Language, Settings, Theme};
use crate::db::DB;
use crate::models::BookDataType;
use crate::types::{BookPath, BookSize};
use itertools::Itertools;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;


impl Default for Settings {
  fn default() -> Self {
    Self {
      id: 1,
      language: Language::EN,
      theme: Theme::Sunset,
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
  pub fn from_pathbuf(book_pathbuf: &PathBuf, book_data_type: BookDataType) -> Self {
    Self {
      path_to_book: book_pathbuf.to_str().unwrap().to_string(),
      path_to_dir: book_pathbuf.parent().unwrap().to_str().unwrap().to_string(),
      book_name: book_pathbuf.file_name().unwrap().to_str().unwrap().to_string(),
      dir_name: book_pathbuf.parent().unwrap().file_name().unwrap().to_str().unwrap().to_string(),
      ext: book_pathbuf.extension().unwrap().to_str().unwrap().to_string(),
      book_data_pk: book_data_type,
      path_is_valid: true,
    }
  }
  pub fn get_all_from_db() -> Vec<Book> {
    let r_conn = DB.r_transaction().unwrap();
    r_conn.scan().primary().unwrap().all().unwrap().try_collect().unwrap()
  }
}

impl DataOfHashedBook {
  pub fn new(hash: String, file_size: BookSize, books_pk: Vec<BookPath>) -> Self {
    DataOfHashedBook {
      book_hash: hash,
      book_size: file_size,
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
impl DataOfUnhashedBook {
  pub fn new(file_size: BookSize, books_pk: Vec<BookPath>) -> Self {
    DataOfUnhashedBook {
      book_size: file_size,
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
  pub fn to_data_of_hashed_book(self, book_hash: String) -> DataOfHashedBook {
    DataOfHashedBook {
      book_hash,
      book_size: self.book_size,
      book_data: self.book_data,
    }
  }
}

pub trait GetBookData {
  fn get_book_data_as_ref(&self) -> &BookData;
}
impl GetBookData for DataOfUnhashedBook {
  fn get_book_data_as_ref(&self) -> &BookData {
    &self.book_data
  }
}
impl GetBookData for DataOfHashedBook {
  fn get_book_data_as_ref(&self) -> &BookData {
    &self.book_data
  }
}

impl Hash for Book {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.path_to_book.hash(state);
  }
}
impl PartialEq for Book {
  fn eq(&self, other: &Self) -> bool {
    self.path_to_book == other.path_to_book
  }
}
