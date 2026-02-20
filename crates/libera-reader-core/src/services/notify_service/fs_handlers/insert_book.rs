use utils::debug;

use crate::{
  db::{
    DB,
    models::books::{
      Books,
      book::{Book, BookPath},
      book_sizes::BookSizes,
    },
  },
  services::NotCachedBooks,
  settings::SETTINGS,
};

pub(crate) async fn insert_book(book_path: BookPath, db: &DB, settings: &SETTINGS, _not_cached_books: &NotCachedBooks) -> anyhow::Result<()> {
  match settings.contains_ext(&book_path.ext) {
    true => {
      let new_book = Book::new(book_path).await.unwrap();
      BookSizes::insert_book(&new_book, db).await.unwrap();
      Books::insert_book(new_book, db).await.unwrap();
    }
    false => {
      debug!("The book with path {:?} has unsupported extension", book_path);
    }
  };
  Ok(())
}
