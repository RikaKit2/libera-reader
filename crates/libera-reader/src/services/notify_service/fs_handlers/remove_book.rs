use std::path::Path;

use native_db::transaction::RwTransaction;

use crate::db::models::books::book::{Book, BookPath};
use crate::db::models::books::book_sizes::BookSizes;
use crate::services::notify_service::RemoveStatus;

pub(crate) fn remove_book_by_path(
  file_path: &Path, rw_t: &RwTransaction<'_>, app_dirs: &crate::app_dirs::AppDirs,
) -> anyhow::Result<RemoveStatus> {
  let Some(book_path) = BookPath::new(file_path) else {
    return Ok(RemoveStatus::FullyDeleted);
  };

  let Some(book) = rw_t.get().primary::<Book>(book_path.clone())? else {
    return Ok(RemoveStatus::FullyDeleted);
  };
  let can_delete = book.can_delete();

  if can_delete {
    // 1. Physically delete book's cache directory (cover and all pages)
    let crate::db::models::books::book::BookSize::BYTES(size) = book.book_size;
    let hash = if let Ok(Some(book_sizes)) = rw_t.get().primary::<BookSizes>(book.book_size) {
      match book_sizes.book_type {
        crate::db::models::books::BookType::DuplicateSize(map) => {
          if let Some(crate::db::models::books::DuplicateBookData::BookHash(h)) = map.get(&book.book_path) {
            Some(h.0.to_string())
          } else {
            None
          }
        }
        _ => None,
      }
    } else {
      None
    };

    let book_cache_dir = app_dirs.book_dir(size, hash.as_deref());
    if book_cache_dir.exists() {
      let _ = std::fs::remove_dir_all(&book_cache_dir);
    }
    let legacy_path = app_dirs.legacy_book_cover_path(size, hash.as_deref());
    if legacy_path.exists() {
      let _ = std::fs::remove_file(&legacy_path);
    }

    // 2. Fully delete the book from BookSizes and Book table
    BookSizes::remove_book(book.book_size, &book.book_path, rw_t)?;
    rw_t.remove::<Book>(book)?;
    Ok(RemoveStatus::FullyDeleted)
  } else {
    // Mark as deleted (soft delete) in BookSizes and Book table
    BookSizes::mark_book_path_as_deleted(book.book_size, &book.book_path, rw_t)?;
    let mut updated_book = book.clone();
    updated_book.mark_as_deleted();
    rw_t.update::<Book>(book, updated_book)?;
    Ok(RemoveStatus::MarkedAsDeleted)
  }
}
