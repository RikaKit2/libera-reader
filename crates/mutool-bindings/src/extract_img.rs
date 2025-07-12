use anyhow::Result;
use std::path::PathBuf;
use utils::FileSizeMeasure;

fn is_file_empty(path: &PathBuf) -> Result<bool> {
  let size = FileSizeMeasure::B.get_file_size(path, 0)?;
  Ok(size == 0.0)
}

pub fn extract_img(path_to_book: &PathBuf) -> Result<()> {
  match is_file_empty(path_to_book)? {
    true => {}
    false => {}
  }
  Ok(())
}
