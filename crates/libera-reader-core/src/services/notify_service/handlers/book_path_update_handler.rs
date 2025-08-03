use crate::db::crud;
use crate::db::models::Book;
use crate::services::notify_service::NotifyEventHandler;
use anyhow::Result;
use std::path::PathBuf;
use tracing::{error, info};


impl NotifyEventHandler {
  pub(crate) fn book_path_update_handler(&mut self, old_path: &PathBuf, new_path: &PathBuf) -> Result<()> {
    let start_time = std::time::Instant::now();
    let res = match crud::get_primary::<Book>(old_path.to_str().unwrap(), &self.db)? {
      None => {
        error!("book_path_update_handler: book not found: {:?}", old_path);
        Ok(self.book_adding_handler(new_path)?)
      }
      Some(book_from_db) => {
        let new_book = Book::from_pathbuf(&new_path, book_from_db.book_data_pk.clone());
        Ok(crud::update(book_from_db, new_book, &self.db)?)
      }
    };
    let total_time = start_time.elapsed();
    info!("Function book_path_update_handler executed in: {:?}", &total_time);
    res
  }
}