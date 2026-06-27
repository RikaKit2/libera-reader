use std::path::Path;

use anyhow::Ok;
use native_db::transaction::RwTransaction;

use crate::db::models::books::book::{Book, BookPath};
use crate::db::models::books::book_sizes::BookSizes;
use crate::services::notify_service::RemoveStatus;

pub(crate) fn remove_book_by_path(
  file_path: &Path, rw_t: &RwTransaction<'_>,
) -> anyhow::Result<RemoveStatus> {
  let Some(book_path) = BookPath::new(file_path) else {
    return Ok(RemoveStatus::FullyDeleted);
  };
  let id = book_path.full_path_string().to_string();

  let Some(book) = rw_t.get().primary::<Book>(id.clone())? else {
    return Ok(RemoveStatus::FullyDeleted);
  };

  let can_delete = book.can_delete();

  // Remove from BookSizes first
  BookSizes::remove_book(book.book_size, &book.book_path, rw_t)?;

  if can_delete {
    // Fully delete the book
    rw_t.remove::<Book>(book)?;
    Ok(RemoveStatus::FullyDeleted)
  } else {
    // Mark as deleted (soft delete) - update book_path.deleted
    let mut updated_book = book.clone();
    updated_book.mark_as_deleted();
    rw_t.update::<Book>(book, updated_book)?;
    Ok(RemoveStatus::MarkedAsDeleted)
  }
}
