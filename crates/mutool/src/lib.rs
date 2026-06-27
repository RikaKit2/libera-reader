#![forbid(unsafe_code)]
pub mod create_book;
pub mod download_mutool;
pub mod extract_img;
pub mod mutool_error;

pub use create_book::create_empty_book;
pub use download_mutool::{download_mutool, download_mutool_if_missing_blocking};
use std::path::{Path, PathBuf};

pub fn get_path_to_mutool(path_to_storage_dir: &Path) -> PathBuf {
  if cfg!(windows) {
    path_to_storage_dir.join("mutool.exe")
  } else {
    path_to_storage_dir.join("mutool")
  }
}
