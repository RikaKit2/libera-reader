use std::fs;
use std::hash::{BuildHasher, Hasher};
use std::io::Read;

use crate::db::models::BookDataType;
use gxhash::GxBuildHasher;


pub(crate) type BookPath = String;
pub(crate) type BookSize = String;
pub(crate) type BookHash = String;
pub(crate) type BooksCount = usize;

pub(crate) struct NotCachedBook {
  pub data_type: BookDataType,
  pub path_to_book: String,
}

fn round_num(x: f64, decimals: u32) -> f64 {
  let y = 10i32.pow(decimals) as f64;
  (x * y).round() / y
}

pub(crate) fn calc_file_size_in_mb(path_to_file: &String) -> f64 {
  let metadata = fs::metadata(path_to_file).unwrap();
  let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
  round_num(size_mb, 6)
}

pub(crate) fn calc_file_hash(path_to_file: &String) -> String {
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
