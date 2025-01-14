use crate::db::crud;
use crate::models::{Book, BookDataType, DataOfHashedBook, DataOfUnhashedBook};
use crate::types::BookPath;
use crate::vars::{APP_DIRS, NOT_CACHED_BOOKS, TARGET_EXT};
use gxhash::GxBuildHasher;
use measure_time_macro::measure_time;
use std::fs;
use std::hash::{BuildHasher, Hasher};
use std::io::Read;
use std::path::PathBuf;
use tracing::debug;
use walkdir::WalkDir;


fn round_num(x: f64, decimals: u32) -> f64 {
  let y = 10i32.pow(decimals) as f64;
  (x * y).round() / y
}

pub(crate) fn calc_file_size_in_mb(path_to_file: &str) -> f64 {
  let metadata = fs::metadata(path_to_file).unwrap();
  let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
  round_num(size_mb, 6)
}

pub(crate) fn calc_file_hash(path_to_file: &PathBuf) -> String {
  let mut hasher = GxBuildHasher::default().build_hasher();
  let mut file = fs::File::open(path_to_file).unwrap();
  loop {
    // Read the file in 1 MB chunks
    let mut buffer = [0; 1024 * 1024];
    let bytes_read = file.read(&mut buffer).unwrap();
    if bytes_read == 0 {
      break;
    }
    hasher.write(&buffer[..bytes_read]);
  }
  data_encoding::HEXLOWER.encode(&hasher.finish().to_ne_bytes())
}

#[measure_time]
pub(crate) fn get_books_from_disk(path_to_scan: &String) -> Vec<PathBuf> {
  let mut books_from_disk: Vec<PathBuf> = vec![];
  for entry in WalkDir::new(path_to_scan) {
    let entry = entry.unwrap();
    if entry.file_type().is_file() {
      let path = entry.path();
      match path.extension() {
        Some(res) => {
          let file_ext = res.to_str().unwrap();
          if TARGET_EXT.read().unwrap().contains(file_ext) {
            books_from_disk.push(path.to_path_buf());
          }
        }
        None => {}
      };
    }
  };
  books_from_disk
}

#[derive(Debug)]
pub(crate) struct NotCachedBook {
  pub book_path: BookPath,
}

impl NotCachedBook {
  pub(crate) fn new(book_path: BookPath) -> Self {
    Self { book_path }
  }
  pub(crate) fn push_to_storage(self) {
    NOT_CACHED_BOOKS.push(self).unwrap();
  }
  pub(crate) fn mark_as_cached(self) {
    let book = crud::get_primary::<Book>(self.book_path).unwrap();
    match book.book_data_pk {
      BookDataType::UniqueSize(book_size) => {
        let old_book_data = crud::get_primary::<DataOfUnhashedBook>(book_size).unwrap();
        let mut new_book_data = old_book_data.clone();
        new_book_data.book_data.cached = true;
        crud::update(old_book_data, new_book_data).unwrap();
      }
      BookDataType::RepeatingSize(book_hash) => {
        let old_book_data = crud::get_primary::<DataOfHashedBook>(book_hash).unwrap();
        let mut new_book_data = old_book_data.clone();
        new_book_data.book_data.cached = true;
        crud::update(old_book_data, new_book_data).unwrap();
      }
    }
  }
  pub(crate) fn get_out_file_name(&self) -> String {
    let book = crud::get_primary::<Book>(self.book_path.clone()).unwrap();
    match &book.book_data_pk {
      BookDataType::UniqueSize(book_size) => {
        APP_DIRS.read().unwrap().dir_of_unhashed_books.join(book_size).to_str().unwrap().to_string()
      }
      BookDataType::RepeatingSize(book_hash) => {
        APP_DIRS.read().unwrap().dir_of_hashed_books.join(book_hash).to_str().unwrap().to_string()
      }
    }
  }
}
