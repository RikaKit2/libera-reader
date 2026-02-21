use std::path::PathBuf;

use utils::debug;

use crate::db::{
  DB,
  models::books::{Books, book::BookDir, book_sizes::BookSizes},
};

pub(crate) fn update_book_dir(old_dir: PathBuf, new_dir: PathBuf, db: &DB) -> anyhow::Result<()> {
  let start_time = std::time::Instant::now();
  let old_book_dir = BookDir::new(old_dir);
  let new_book_dir = BookDir::new(new_dir);
  if let Some(old_books) = Books::get_by_parent_dir(old_book_dir, db)? {
    let mut updated_books = old_books.clone();
    updated_books.parent_dir = new_book_dir.clone();
    for (_book_name, book) in updated_books.storage.iter_mut() {
      let old_book_path = book.book_path.clone();
      book.book_path.parent_dir = new_book_dir.clone();
      BookSizes::update_book_path(book.book_size, &old_book_path, book.book_path.clone(), db)?;
    }
    db.update(old_books, updated_books)?;
  };

  let total_time = start_time.elapsed();
  debug!("The total time to update parent dir of the book: {:?}", &total_time);
  Ok(())
}
