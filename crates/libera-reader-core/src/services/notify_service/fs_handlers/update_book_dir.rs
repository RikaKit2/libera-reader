use std::path::PathBuf;

use tracing::info;

use crate::db::{
  DB,
  models::books::{Books, book::BookDir, book_sizes::BookSizes},
};

pub(crate) fn update_book_dir(old_dir: PathBuf, new_dir: PathBuf, db: &DB) -> anyhow::Result<()> {
  let start_time = std::time::Instant::now();
  let old_book_dir = BookDir::new(old_dir);
  let new_book_dir = BookDir::new(new_dir);
  match Books::get_by_parent_dir(old_book_dir, db)? {
    Some(old_books) => {
      let mut updated_books = old_books.clone();
      updated_books.parent_dir = new_book_dir.clone();
      for (_book_name, book) in updated_books.storage.iter_mut() {
        let old_book_path = book.book_path.clone();
        book.book_path.parent_dir = new_book_dir.clone();
        BookSizes::update_book_path(book.book_size.clone(), &old_book_path, book.book_path.clone(), db)?;
      }
      db.update(old_books, updated_books)?;
    }
    None => {}
  };

  let total_time = start_time.elapsed();
  info!("The total time to update parent dir of the book: {:?}", &total_time);
  Ok(())
}
