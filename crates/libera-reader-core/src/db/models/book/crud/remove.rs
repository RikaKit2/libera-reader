use std::path::PathBuf;

use utils::get_file_size;

use crate::db::{
  DB,
  models::{HashedBooks, UniqueBook},
};

pub(crate) async fn remove(pathbuf: &PathBuf, db: &DB) -> anyhow::Result<()> {
  let book_path = pathbuf.to_str().unwrap().to_string();
  let book_size = get_file_size(pathbuf).await?;
  let hashed_books_with_such_size = HashedBooks::scan_by_size(book_size, db)?;
  match hashed_books_with_such_size.len() > 0 {
    true => {
      for hashed_books in hashed_books_with_such_size {
        match hashed_books.remove_by_path(&book_path, db)? {
          true => {
            break;
          }
          false => {}
        }
      }
    }
    false => {
      let book = UniqueBook::get_by_path(book_path.clone(), db)?.unwrap();
      book.remove(db)?;
    }
  };
  Ok(())
}
