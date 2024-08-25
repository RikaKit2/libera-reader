use std::fs;
use std::hash::{BuildHasher, Hasher};
use std::io::Read;
use std::path::PathBuf;

use gxhash::GxBuildHasher;

use crate::db::model::book_item;

pub struct NotCachedBook {
  pub book_hash: String,
  pub path_to_book: String,
}

fn round_num(x: f64, decimals: u32) -> f64 {
  let y = 10i32.pow(decimals) as f64;
  (x * y).round() / y
}

pub fn calc_file_size_in_mb(path_to_file: &String) -> f64 {
  let metadata = fs::metadata(path_to_file).unwrap();
  let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
  round_num(size_mb, 8)
}

pub struct BookHashAndSize {
  pub book_hash: String,
  pub book_size: f64,
}

pub struct BookData {
  pub path_to_book: String,
  pub path_to_dir: String,
  pub book_name: String,
  pub dir_name: String,
  pub ext: String,
}

impl BookData {
  pub fn from_pathbuf(pathbuf: &PathBuf) -> Self {
    let path_to_book = pathbuf.to_str().unwrap().to_string();
    Self {
      path_to_book,
      path_to_dir: pathbuf.parent().unwrap().to_str().unwrap().to_string(),
      book_name: pathbuf.file_name().unwrap().to_str().unwrap().to_string(),
      dir_name: pathbuf.parent().unwrap().file_name().unwrap().to_str().unwrap().to_string(),
      ext: pathbuf.extension().unwrap().to_str().unwrap().to_string(),
    }
  }
  pub fn from_book_item(book_item: book_item::Data) -> Self {
    let path_to_book = book_item.path_to_book;
    Self {
      path_to_book,
      path_to_dir: book_item.path_to_dir,
      book_name: book_item.book_name,
      dir_name: book_item.dir_name,
      ext: book_item.ext,
    }
  }
}

pub fn calc_file_hash(path_to_file: &String) -> String {
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

