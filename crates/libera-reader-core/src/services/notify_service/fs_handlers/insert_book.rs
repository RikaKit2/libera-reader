use native_db::transaction::RwTransaction;
use utils::debug;

use crate::{
  db::models::books::{
    Books,
    book::{Book, BookPath},
    book_sizes::BookSizes,
  },
  services::NotCachedBooks,
  types::MUPDF_EXTENSIONS,
};

pub(crate) fn insert_book(
  book_path: BookPath, rw_t: &RwTransaction<'_>, _not_cached_books: &NotCachedBooks,
) -> anyhow::Result<()> {
  match MUPDF_EXTENSIONS.contains(&book_path.ext.to_string().as_str()) {
    true => {
      let new_book = Book::new(book_path).unwrap();
      BookSizes::insert_book(&new_book, rw_t).unwrap();
      Books::insert_book(new_book, rw_t).unwrap();
    }
    false => {
      debug!("The book with path {:?} has unsupported extension", book_path);
    }
  };
  Ok(())
}
