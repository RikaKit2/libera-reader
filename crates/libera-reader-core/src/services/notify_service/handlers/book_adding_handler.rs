use crate::{
  db::{DB, models::Book},
  settings::Settings,
  types::NotCachedBooks,
};
use anyhow::Result;
use std::path::PathBuf;
use tracing::info;

pub(crate) async fn book_adding_handler(book_pathbuf: &PathBuf, settings: &Settings, not_cached_books: &NotCachedBooks, db: &DB) -> Result<()> {
  let start_time = std::time::Instant::now();
  let ext = book_pathbuf.extension().unwrap().to_str().unwrap().to_string();
  if settings.contains_ext(&ext) {
    let book_size = utils::get_file_size(book_pathbuf).await?;
    Book::insert(book_pathbuf, book_size, db, not_cached_books).await?;
  }
  let total_time = start_time.elapsed();
  info!("Function book_adding_handler executed in: {:?}", &total_time);
  Ok(())
}
