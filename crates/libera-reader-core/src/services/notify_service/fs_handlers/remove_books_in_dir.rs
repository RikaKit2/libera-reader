use crate::db::models::books::{Books, book::BookDir};
use native_db::transaction::RwTransaction;

pub(crate) fn remove_books_in_dir(
  book_dir: BookDir, rw_t: &RwTransaction<'_>,
) -> anyhow::Result<()> {
  if let Some(old_books) = Books::get_by_parent_dir_rw(book_dir, rw_t)? {
    old_books.remove_self(rw_t)?;
  };
  Ok(())
}
