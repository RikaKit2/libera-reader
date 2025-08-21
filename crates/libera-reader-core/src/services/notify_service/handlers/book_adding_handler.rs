use crate::db::models::Book;
use crate::services::notify_service::NotifyEventHandler;
use anyhow::Result;
use std::path::PathBuf;
use tracing::info;
use utils::get_file_size;

impl NotifyEventHandler {
  //noinspection RsUnwrap
  pub(crate) fn book_adding_handler(&self, book_pathbuf: &PathBuf) -> Result<()> {
    let start_time = std::time::Instant::now();
    let ext = book_pathbuf.extension().unwrap().to_str().unwrap().to_string();
    if self.settings.contains_ext(&ext) {
      let book_size = get_file_size(book_pathbuf)?;
      Book::insert_to_db(book_pathbuf, book_size, &self.db, &self.not_cached_books);
    }
    let total_time = start_time.elapsed();
    info!("Function book_adding_handler executed in: {:?}", &total_time);
    Ok(())
  }
}