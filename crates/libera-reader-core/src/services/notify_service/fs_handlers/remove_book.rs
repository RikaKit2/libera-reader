use std::path::Path;

use anyhow::Ok;
use native_db::transaction::RwTransaction;

use crate::db::models::books::Books;
use crate::db::models::books::book::BookDir;

pub(crate) fn remove_book_by_path(file_path: &Path, rw_t: &RwTransaction<'_>) -> anyhow::Result<()> {
  let Some(parent_dir) = file_path.parent() else {
    return Ok(());
  };
  let Some(file_name) = file_path.file_name() else {
    return Ok(());
  };

  let parent_dir = BookDir::new(parent_dir.to_path_buf());
  let file_name_str = file_name.to_string_lossy().to_string();

  if let Some(books) = Books::get_by_parent_dir_rw(parent_dir, rw_t)? {
    for (book_name, book) in &books.storage {
      let full_name = format!("{}.{}", book_name, book.book_path.ext);
      if full_name == file_name_str {
        return books.remove_book(book.book_path.clone(), rw_t);
      }
    }
  }

  Ok(())
}
