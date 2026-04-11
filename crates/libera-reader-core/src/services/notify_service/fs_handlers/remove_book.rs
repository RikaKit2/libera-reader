use std::path::Path;

use anyhow::Ok;
use native_db::transaction::RwTransaction;

use crate::db::models::books::Books;
use crate::db::models::books::book::BookDir;
use crate::services::notify_service::RemoveStatus;

pub(crate) fn remove_book_by_path(file_path: &Path, rw_t: &RwTransaction<'_>) -> anyhow::Result<RemoveStatus> {
  let Some(parent_dir) = file_path.parent() else {
    return Ok(RemoveStatus::FullyDeleted);
  };
  let Some(file_name) = file_path.file_name() else {
    return Ok(RemoveStatus::FullyDeleted);
  };

  let parent_dir = BookDir::new(parent_dir.to_path_buf());
  let file_name_str = file_name.to_string_lossy().to_string();

  if let Some(books) = Books::get_by_parent_dir_rw(parent_dir, rw_t)? {
    for (book_name, book) in &books.storage {
      let full_name = format!("{}.{}", book_name, book.book_path.ext);
      if full_name == file_name_str {
        let can_delete = book.can_delete();

        if can_delete {
          // Book will be fully deleted
          books.remove_book(book.book_path.clone(), rw_t)?;
          return Ok(RemoveStatus::FullyDeleted);
        } else {
          // Book will be marked as deleted (soft delete)
          books.remove_book(book.book_path.clone(), rw_t)?;
          return Ok(RemoveStatus::MarkedAsDeleted);
        }
      }
    }
  }

  // Book not found in DB - treat as already deleted
  Ok(RemoveStatus::FullyDeleted)
}
