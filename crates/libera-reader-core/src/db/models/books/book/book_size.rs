use native_db::*;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use utils::error;

const MB_BYTES: u64 = 1024 * 1024;
const MAX_FILE_SIZE_MB: u64 = 100;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Eq, PartialEq)]
pub enum BookSize {
  BYTES(u64),
}
impl BookSize {
  pub(crate) async fn new(path_to_file: &PathBuf) -> anyhow::Result<Self> {
    match path_to_file.is_file() {
      true => match tokio::fs::metadata(path_to_file).await {
        Ok(metadata) => {
          let file_size = metadata.len();
          Ok(BookSize::BYTES(file_size))
        }
        Err(err) => {
          error!("{:?}", &path_to_file);
          panic!("{:?}", &err);
        }
      },
      false => {
        panic!("is not a file: {:?}", path_to_file);
      }
    }
  }
  pub fn as_mb(&self) -> u64 {
    match &self {
      BookSize::BYTES(size) => size / MB_BYTES,
    }
  }
  pub fn need_for_hashing(&self) -> bool {
    match &self {
      BookSize::BYTES(size) => {
        let file_size_mb = size / MB_BYTES;
        file_size_mb <= MAX_FILE_SIZE_MB
      }
    }
  }
}
impl ToKey for BookSize {
  fn to_key(&self) -> Key {
    match &self {
      BookSize::BYTES(size) => Key::new(size.to_string().as_bytes().to_vec()),
    }
  }
  fn key_names() -> Vec<String> {
    vec!["BookSize".to_string()]
  }
}
