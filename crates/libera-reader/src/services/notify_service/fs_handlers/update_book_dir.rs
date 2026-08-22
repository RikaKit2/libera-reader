use native_db::transaction::RwTransaction;
use std::path::PathBuf;

use itertools::Itertools;
use utils::debug;

use crate::db::models::books::{
  book::{Book, BookDir},
  book_sizes::BookSizes,
};

pub(crate) fn update_book_dir(
  old_dir: PathBuf, new_dir: PathBuf, rw_t: &RwTransaction<'_>,
) -> anyhow::Result<()> {
  let start_time = std::time::Instant::now();
  let old_book_dir = BookDir::new(old_dir);
  let new_book_dir = BookDir::new(new_dir);
  let old_dir_path = old_book_dir.full_path().to_string();
  let new_dir_path = new_book_dir.full_path().to_string();

  // Scan ALL books and filter by parent_dir
  let all_books: Vec<Book> = rw_t.scan().primary::<Book>()?.all()?.try_collect()?;
  let books_in_dir: Vec<Book> =
    all_books.into_iter().filter(|b| b.parent_dir == old_dir_path).collect();

  for book in books_in_dir {
    let old_book_path = book.book_path.clone();
    let mut updated_book = book.clone();

    // Update parent_dir in Book
    updated_book.parent_dir = new_dir_path.clone();
    updated_book.book_path.parent_dir = new_book_dir.clone();

    // Update parent_dir in the id (full path string)
    let new_full_path = updated_book.book_path.as_pathbuf();
    updated_book.id = new_full_path.to_string_lossy().to_string();

    // Update BookSizes path
    BookSizes::update_book_path(
      book.book_size,
      &old_book_path,
      updated_book.book_path.clone(),
      rw_t,
    )?;

    // Update in DB - need to remove old and insert new since primary key changed
    rw_t.remove::<Book>(book)?;
    rw_t.insert::<Book>(updated_book)?;
  }

  let total_time = start_time.elapsed();
  debug!("The total time to update parent dir of the book: {:?}", &total_time);
  Ok(())
}
