use crate::db::crud;
use crate::db::models::Book;
use crate::services::notify_service::NotifyEventHandler;
use anyhow::Result;
use tracing::info;

impl NotifyEventHandler {
  pub(crate) fn book_deletion_handler(&self, path_to_book: &str) -> Result<()> {
    let start_time = std::time::Instant::now();
    match crud::get_primary::<Book>(path_to_book, &self.db)? {
      None => { info!("book_deletion_handler: book not found: {path_to_book}") }
      Some(old_book) => { Book::remove(old_book, &self.db)?; }
    };
    let total_time = start_time.elapsed();
    info!("Function book_deletion_handler executed in: {:?}", &total_time);
    Ok(())
  }
}