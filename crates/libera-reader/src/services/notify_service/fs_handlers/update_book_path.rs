use native_db::transaction::RwTransaction;
use std::path::PathBuf;

use crate::utils::debug;
use anyhow::Ok;

use crate::db::models::books::{
  book::{Book, BookPath},
  book_sizes::BookSizes,
};

pub(crate) fn update_book_path(
  old_path: PathBuf, new_path: PathBuf, rw_t: &RwTransaction<'_>,
) -> anyhow::Result<()> {
  let start_time = std::time::Instant::now();
  let old_book_path = BookPath::new(&old_path);
  let new_book_path = BookPath::new(&new_path);

  match old_book_path {
    Some(old_book_path) => match new_book_path {
      Some(new_book_path) => {
        if let Some(book) = rw_t.get().primary::<Book>(old_book_path.clone())? {
          let mut updated_book = book.clone();
          updated_book.book_path = new_book_path.clone();
          updated_book.parent_dir = new_book_path.parent_dir.clone();

          BookSizes::update_book_path(book.book_size, &old_book_path, new_book_path, rw_t)?;

          // Remove old key and insert new since primary key changed
          rw_t.remove::<Book>(book)?;
          rw_t.insert::<Book>(updated_book)?;

          let total_time = start_time.elapsed();
          debug!("The total time to update the path of the book: {:?}", &total_time);
        }
      }
      None => debug!("The new path is not a valid book path: {:?}", &new_path),
    },
    None => debug!("The old path is not a valid book path: {:?}", &old_path),
  }
  Ok(())
}
