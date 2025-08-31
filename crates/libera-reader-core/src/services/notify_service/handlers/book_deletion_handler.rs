use crate::db::{DB, models::Book};
use anyhow::Result;
use tracing::info;

pub(crate) fn book_deletion_handler(path_to_book: &str, db: &DB) -> Result<()> {
  let start_time = std::time::Instant::now();
  match db.get_primary::<Book>(path_to_book)? {
    None => {
      info!("book_deletion_handler: book not found: {path_to_book}")
    }
    Some(old_book) => {
      Book::remove(old_book, db)?;
    }
  };
  let total_time = start_time.elapsed();
  info!("Function book_deletion_handler executed in: {:?}", &total_time);
  Ok(())
}
