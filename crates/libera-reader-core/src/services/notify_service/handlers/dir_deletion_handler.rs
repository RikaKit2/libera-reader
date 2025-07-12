use crate::db::crud;
use crate::services::notify_service::NotifyEventHandler;
use anyhow::Result;

impl NotifyEventHandler {
  pub(crate) fn dir_deletion_handler(&self, path_to_dir: String) -> Result<()> {
    for old_book in crud::book::get_books_located_in_dir(path_to_dir, &self.db)? {
      self.book_deletion_handler(old_book.path_to_book.as_str())?;
    }
    Ok(())
  }
}