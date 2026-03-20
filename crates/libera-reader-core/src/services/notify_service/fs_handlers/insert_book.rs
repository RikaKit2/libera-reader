use native_db::transaction::RwTransaction;
use utils::debug;

use crate::{
  db::models::books::{
    Books,
    book::{Book, BookPath},
    book_sizes::BookSizes,
  },
  services::NotCachedBooks,
  settings::SETTINGS,
};

pub(crate) fn insert_book(
  book_path: BookPath, rw_t: &RwTransaction<'_>, settings: &SETTINGS, _not_cached_books: &NotCachedBooks,
) -> anyhow::Result<()> {
  match settings.contains_ext(&book_path.ext) {
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
