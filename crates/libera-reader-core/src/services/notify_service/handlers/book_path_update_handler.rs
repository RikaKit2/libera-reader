use crate::{
  db::{DB, models::Book},
  services::notify_service::handlers::book_adding_handler,
  settings::Settings,
  types::NotCachedBooks,
};
use anyhow::Result;
use std::path::PathBuf;
use tracing::{error, info};

pub(crate) async fn book_path_update_handler(
  old_path: &PathBuf, new_path: &PathBuf, settings: &Settings, not_cached_books: &NotCachedBooks, db: &DB,
) -> Result<()> {
  let start_time = std::time::Instant::now();
  let res = match db.get_primary::<Book>(old_path.to_str().unwrap())? {
    None => {
      error!("book_path_update_handler: book not found: {:?}", old_path);
      Ok(book_adding_handler(new_path, settings, not_cached_books, db).await?)
    }
    Some(book_from_db) => {
      let new_book = Book::from_pathbuf(&new_path, book_from_db.book_data_pk.clone());
      Ok(db.update(book_from_db, new_book)?)
    }
  };
  let total_time = start_time.elapsed();
  info!("Function book_path_update_handler executed in: {:?}", &total_time);
  res
}
