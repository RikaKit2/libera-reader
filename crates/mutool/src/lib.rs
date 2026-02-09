pub mod create_book;
pub mod download_mutool;
pub mod extract_img;
pub mod mutool_status;

pub use create_book::create_empty_book;
pub use download_mutool::{download_mutool, download_mutool_if_missing_blocking};
use std::path::PathBuf;

pub fn get_path_to_mutool(path_to_storage_dir: &PathBuf) -> PathBuf {
  if cfg!(windows) { path_to_storage_dir.join("mutool.exe") } else { path_to_storage_dir.join("mutool") }
}
