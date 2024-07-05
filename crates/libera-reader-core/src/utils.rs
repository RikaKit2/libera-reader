use std::fs;
use std::io::Read;

use blake3;

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
  round_num(size_mb, 2)
}


pub fn calc_blake3_hash_of_file(path_to_file: &String) -> String {
  let mut file = fs::File::open(path_to_file).unwrap();
  let mut hasher = blake3::Hasher::new();
  loop {
    // Read the file in 4KB chunks
    let mut buffer = [0; 4096];
    let bytes_read = file.read(&mut buffer).unwrap();
    if bytes_read == 0 {
      break;
    }
    hasher.update(&buffer[..bytes_read]);
  }
  hasher.finalize().to_hex().to_string()
}

