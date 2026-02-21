use crate::db::{
  DB,
  models::books::{Books, book::BookDir},
};

pub(crate) fn remove_books_in_dir(book_dir: BookDir, db: &DB) -> anyhow::Result<()> {
  if let Some(old_books) = Books::get_by_parent_dir(book_dir, db)? {
    old_books.remove_self(db)?;
  };
  Ok(())
}
