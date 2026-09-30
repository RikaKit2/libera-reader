use crate::db::models::books::book::{Book, BookDir, BookSize};
use crate::db::models::books::book_sizes::BookSizes;
use itertools::Itertools;
use native_db::transaction::RwTransaction;

pub(crate) fn remove_books_in_dir(
  book_dir: BookDir, rw_t: &RwTransaction<'_>, app_dirs: &crate::app_dirs::AppDirs,
) -> anyhow::Result<()> {
  // Scan ALL books and filter by parent_dir
  let all_books: Vec<Book> = rw_t.scan().primary::<Book>()?.all()?.try_collect()?;
  let books_to_process: Vec<Book> =
    all_books.into_iter().filter(|b| b.parent_dir == book_dir).collect();

  for book in books_to_process {
    let can_delete = book.can_delete();
    if can_delete {
      let BookSize::BYTES(size) = book.book_size;
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

      BookSizes::remove_book(book.book_size, &book.book_path, rw_t)?;
      rw_t.remove::<Book>(book)?;
    } else {
      BookSizes::mark_book_path_as_deleted(book.book_size, &book.book_path, rw_t)?;
      let mut updated_book = book.clone();
      updated_book.mark_as_deleted();
      rw_t.update::<Book>(book, updated_book)?;
    }
  }

  Ok(())
}
