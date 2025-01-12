use crate::db::models::{Book, BookData, DataOfHashedBook, DataOfUnhashedBook,
                        Language, Settings, Theme};
use crate::db::{crud, DB};
use crate::models::{BookDataType, DataOfHashedBookKey, TargetExt};
use crate::services::notify_service;
use crate::types::{BookHash, BookPath, BookSize};
use crate::vars::PATH_TO_SCAN;
use itertools::Itertools;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use tracing::error;


impl Book {
  pub(crate) fn from_pathbuf(future_book: &PathBuf, book_data_type: BookDataType) -> Self {
    Self {
      path_to_book: future_book.to_str().unwrap().to_string(),
      path_to_dir: future_book.parent().unwrap().to_str().unwrap().to_string(),
      book_name: future_book.file_name().unwrap().to_str().unwrap().to_string(),
      dir_name: future_book.parent().unwrap().file_name().unwrap().to_str().unwrap().to_string(),
      ext: future_book.extension().unwrap().to_str().unwrap().to_string(),
      book_data_pk: book_data_type,
      path_is_valid: true,
    }
  }
  pub(crate) fn get_all_from_db() -> Vec<Book> {
    let r_conn = DB.r_transaction().unwrap();
    r_conn.scan().primary().unwrap().all().unwrap().try_collect().unwrap()
  }
  pub(crate) fn get_num_of_books_of_this_size(book_size: BookSize) -> (usize, Option<DataOfUnhashedBook>) {
    let mut out_data: Option<DataOfUnhashedBook> = None;
    let mut num_of_book_with_this_size = 0;
    match crud::get_primary::<DataOfUnhashedBook>(book_size.clone()) {
      None => {
        let r_conn = DB.r_transaction().unwrap();
        for i in r_conn.scan().secondary::<DataOfHashedBook>(DataOfHashedBookKey::book_size).unwrap().all().unwrap() {
          match i {
            Ok(_data) => { num_of_book_with_this_size += 1; }
            Err(_) => {}
          }
        }
      }
      Some(data) => {
        num_of_book_with_this_size = data.book_data.books_pk.len();
        out_data = Some(data);
      }
    };
    (num_of_book_with_this_size, out_data)
  }
  pub(crate) fn update_book_data_type(book_path: BookPath, book_data_type: BookDataType) {
    let old_book = crud::get_primary::<Book>(book_path).unwrap();
    let mut new_book = old_book.clone();
    new_book.book_data_pk = book_data_type;
    crud::update(old_book, new_book).unwrap();
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
impl Eq for Book {}

impl Settings {
  pub(crate) fn new() -> Settings {
    *PATH_TO_SCAN.write().unwrap() = Settings::get_self().path_to_scan;
    Self::get_self()
  }
  pub fn set_path_to_scan(&mut self, path_to_scan: String) {
    let old_settings = Settings::get_self();
    match &old_settings.path_to_scan {
      None => {}
      Some(path_to_scan) => notify_service::stop_watcher(path_to_scan)
    };
    notify_service::run_watcher(&path_to_scan);
    let mut new_settings = old_settings.clone();
    new_settings.path_to_scan = Some(path_to_scan.clone());
    match crud::update::<Self>(old_settings, new_settings) {
      Ok(_) => {}
      Err(e) => {
        error!("{:?}", &e);
        panic!("{:?}", e);
      }
    };
    let _ = PATH_TO_SCAN.write().unwrap().insert(path_to_scan);
  }
  pub fn get_path_to_scan(&self) -> Option<String> {
    PATH_TO_SCAN.read().unwrap().clone()
  }
  pub fn path_to_scan_is_valid(&self) -> bool {
    PATH_TO_SCAN.read().unwrap().is_some()
  }
  //noinspection RsUnwrap
  fn get_self() -> Self {
    match crud::get_primary::<Self>(1) {
      None => {
        let settings_model = Self::default();
        crud::insert(settings_model.clone()).unwrap();
        settings_model
      }
      Some(res) => { res }
    }
  }
}
impl Default for Settings {
  fn default() -> Self {
    Self {
      id: 1,
      language: Language::EN,
      theme: Theme::Sunset,
      path_to_scan: None,
      number_of_columns: 6,
      page_scaling_factor: 1.0,
      thumbnails_scaling_factor: 4.0,
      workers_num: 2,
    }
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
  pub(crate) fn replace_to_data_of_hashed_book(self, book_hash: BookHash) {
    let old_book_data = crud::remove::<Self>(self).unwrap();
    let new_book_data = DataOfHashedBook {
      book_hash,
      book_size: old_book_data.book_size,
      book_data: old_book_data.book_data,
    };
    crud::insert::<DataOfHashedBook>(new_book_data).unwrap();
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

impl TargetExt {
  pub(crate) fn new() -> Self {
    Self {
      id: 1,
      pdf: true,
      epub: false,
      mobi: false,
    }
  }
  pub(crate) fn from_db() -> Option<TargetExt> {
    crud::get_primary::<TargetExt>(1)
  }
  pub fn contains(&self, ext: &str) -> bool {
    let ext_is_pdf = ext.eq("pdf") && self.pdf;
    let ext_is_epub = ext.eq("epub") && self.epub;
    let ext_is_mobi = ext.eq("mobi") && self.mobi;
    if ext_is_pdf || ext_is_epub || ext_is_mobi {
      true
    } else {
      false
    }
  }
  pub fn set_pdf(&mut self, value: bool) {
    let old_self = Self::get_self();
    let mut new_self = old_self.clone();
    new_self.pdf = value.clone();
    crud::update(old_self, new_self).unwrap();
    self.pdf = value;
  }
  pub fn set_epub(&mut self, value: bool) {
    let old_self = Self::get_self();
    let mut new_self = old_self.clone();
    new_self.epub = value.clone();
    crud::update(old_self, new_self).unwrap();
    self.epub = value;
  }
  pub fn set_mobi(&mut self, value: bool) {
    let old_self = Self::get_self();
    let mut new_self = old_self.clone();
    new_self.mobi = value.clone();
    crud::update(old_self, new_self).unwrap();
    self.mobi = value;
  }
  pub(crate) fn get_self() -> Self {
    crud::get_primary::<Self>(1).unwrap()
  }
}
impl Default for TargetExt {
  fn default() -> Self {
    match TargetExt::from_db() {
      None => {
        crud::insert::<Self>(TargetExt::new()).unwrap();
        TargetExt::from_db().unwrap()
      }
      Some(res) => {
        res
      }
    }
  }
}

