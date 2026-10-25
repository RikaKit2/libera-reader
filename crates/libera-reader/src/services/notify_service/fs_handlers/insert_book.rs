use crate::utils::debug;
use native_db::transaction::RwTransaction;

use crate::{
  db::models::books::{
    book::{Book, BookPath},
    book_sizes::BookSizes,
  },
  not_cached_books::NotCachedBooks,
  types::MUPDF_EXTENSIONS,
};

pub(crate) fn insert_book(
  book_path: BookPath, rw_t: &RwTransaction<'_>, not_cached_books: &NotCachedBooks,
  app_dirs: &crate::app_dirs::AppDirs,
) -> anyhow::Result<()> {
  match MUPDF_EXTENSIONS.contains(&book_path.ext.to_string().as_str()) {
    true => {
      let new_book = Book::new(book_path)?;
      let transitioned = BookSizes::insert_book(&new_book, rw_t)?;
      if let Some(prev_unique_path) = transitioned {
        let crate::db::models::books::book::BookSize::BYTES(size) = new_book.book_size;
        let unhashed_dir = app_dirs.book_dir(size, None);
        if unhashed_dir.exists() {
          let _ = std::fs::remove_dir_all(&unhashed_dir);
        }
        let legacy_path = app_dirs.legacy_book_cover_path(size, None);
        if legacy_path.exists() {
          let _ = std::fs::remove_file(&legacy_path);
        }
        let _ = not_cached_books.tx().send(prev_unique_path);
      }
      rw_t.insert::<Book>(new_book)?;
    }
    false => {
      debug!("The book with path {:?} has unsupported extension", book_path);
    }
  };
  Ok(())
}
