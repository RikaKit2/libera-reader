use crate::db::crud;
use crate::db::models::Book;
use crate::services::notify_service::NotifyEventHandler;
use anyhow::Result;
use tracing::debug;

impl NotifyEventHandler {
  pub(crate) fn book_deletion_handler(&self, path_to_book: &str) -> Result<()> {
    let start_time = std::time::Instant::now();
    match crud::get_primary::<Book>(path_to_book, &self.db)? {
      None => { debug!("book_deletion_handler: book not found: {path_to_book}") }
      Some(old_book) => { crud::book::del_book(old_book, &self.app_dirs, &self.db)?; }
    };
    let total_time = start_time.elapsed();
    debug!("Function book_deletion_handler executed in: {:?}", &total_time);
    Ok(())
  }
}