use std::path::PathBuf;

use anyhow::Ok;
use utils::debug;

use crate::db::{
  DB,
  models::books::{Books, book::BookPath, book_sizes::BookSizes},
};

pub(crate) async fn update_book_path(old_path: PathBuf, new_path: PathBuf, db: &DB) -> anyhow::Result<()> {
  let start_time = std::time::Instant::now();
  let old_book_path = BookPath::new(&old_path);
  let new_book_path = BookPath::new(&new_path);

  match old_book_path {
    Some(old_book_path) => match new_book_path {
      Some(new_book_path) => {
        if let Some(old_books) = Books::get_by_parent_dir(old_book_path.parent_dir.clone(), db)? {
          let mut updated_books = old_books.clone();

          if let Some(mut book) = updated_books.storage.swap_remove(&old_book_path.name) {
            book.book_path = new_book_path.clone();
            BookSizes::update_book_path(book.book_size, &old_book_path, new_book_path, db)?;
            updated_books.parent_dir = book.book_path.parent_dir.clone();
            updated_books.storage.insert(book.book_path.name.clone(), book);

            db.update(old_books, updated_books)?;
            let total_time = start_time.elapsed();
            debug!("The total time to update the path of the book: {:?}", &total_time);
          };
        };
      }
      None => debug!("The new path is not a valid book path: {:?}", &new_path),
    },
    None => debug!("The old path is not a valid book path: {:?}", &old_path),
  }
  Ok(())
}
