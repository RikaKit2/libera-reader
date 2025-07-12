use crate::db::crud;
use crate::db::models::Book;
use crate::types::{APP_DIRS, DB};
use anyhow::Result;
use measure_time_macro::measure_time;
use tracing::debug;

#[measure_time]
pub(crate) fn del_outdated_books(outdated_books: Vec<Book>, db: &DB, app_dirs: &APP_DIRS) -> Result<()> {
  for outdated_book in outdated_books {
    crud::book::del_book(outdated_book, app_dirs, db)?;
  }
  Ok(())
}