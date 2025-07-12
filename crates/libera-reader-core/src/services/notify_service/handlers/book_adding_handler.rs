use crate::db::crud;
use crate::services::notify_service::NotifyEventHandler;
use anyhow::Result;
use std::path::PathBuf;
use tracing::debug;
use utils::FileSizeMeasure;

impl NotifyEventHandler {
  //noinspection RsUnwrap
  pub(crate) fn book_adding_handler(&self, book_pathbuf: &PathBuf) -> Result<()> {
    let start_time = std::time::Instant::now();
    let ext = book_pathbuf.extension().unwrap().to_str().unwrap().to_string();
    if self.target_ext.read().unwrap().contains(&ext) {
      let book_size = FileSizeMeasure::MB.get_file_size(book_pathbuf, 2)?.to_string();
      crud::book::add_book(book_pathbuf, book_size, &self.db, &self.not_cached_books)?;
    }
    let total_time = start_time.elapsed();
    debug!("Function book_adding_handler executed in: {:?}", &total_time);
    Ok(())
  }
}