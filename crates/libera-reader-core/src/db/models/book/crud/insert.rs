use std::path::PathBuf;

use crate::{
  db::{
    DB,
    models::{HashedBooks, UniqueBook},
  },
  types::{BookSize, NotCachedBooks},
};

pub(crate) async fn insert(pathbuf: &PathBuf, book_size: BookSize, db: &DB, not_cached_books: &NotCachedBooks) -> anyhow::Result<()> {
  let hashed_books_count: usize = HashedBooks::scan_by_size(book_size, db)?.len();
  match hashed_books_count > 0 {
    true => {
      HashedBooks::insert(pathbuf, db, book_size, not_cached_books).await?;
    }
    false => match UniqueBook::get_by_size(book_size, db)? {
      Some(unique_book) => {
        db.remove(unique_book.clone())?;
        HashedBooks::insert_using_unique_book(unique_book, db, not_cached_books).await?;
      }
      None => {
        UniqueBook::insert(pathbuf, book_size.clone(), db, not_cached_books).await?;
      }
    },
  };
  Ok(())
}
