use crate::{
  db::{DB, models::Book},
  services::notify_service::handlers::book_deletion_handler,
};
use anyhow::Result;

pub(crate) fn dir_deletion_handler(path_to_dir: String, db: &DB) -> Result<()> {
  for old_book in Book::get_books_located_in_dir(path_to_dir, db)? {
    book_deletion_handler(old_book.full_path.as_str(), db)?;
  }
  Ok(())
}
