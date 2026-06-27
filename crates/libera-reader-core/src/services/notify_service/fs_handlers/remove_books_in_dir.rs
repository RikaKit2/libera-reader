use crate::db::models::books::book::{Book, BookDir};
use crate::db::models::books::book_sizes::BookSizes;
use itertools::Itertools;
use native_db::transaction::RwTransaction;

pub(crate) fn remove_books_in_dir(
  book_dir: BookDir, rw_t: &RwTransaction<'_>,
) -> anyhow::Result<()> {
  let dir_path = book_dir.full_path().to_string();

  // Scan ALL books and filter by parent_dir (native_db secondary keys are internal types)
  let all_books: Vec<Book> = rw_t.scan().primary::<Book>()?.all()?.try_collect()?;
  let books_to_process: Vec<Book> =
    all_books.into_iter().filter(|b| b.parent_dir == dir_path).collect();

  for book in books_to_process {
    let can_delete = book.can_delete();
    BookSizes::remove_book(book.book_size, &book.book_path, rw_t)?;

    if can_delete {
      rw_t.remove::<Book>(book)?;
    } else {
      let mut updated_book = book.clone();
      updated_book.mark_as_deleted();
      rw_t.update::<Book>(book, updated_book)?;
    }
  }

  Ok(())
}
