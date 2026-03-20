use anyhow::Ok;
use native_db::transaction::RwTransaction;

use crate::db::models::books::{Books, book::BookPath};

pub(crate) fn remove_book(book_path: BookPath, rw_t: &RwTransaction<'_>) -> anyhow::Result<()> {
  if let Some(old_self) = Books::get_by_parent_dir_rw(book_path.parent_dir.clone(), rw_t)? {
    old_self.remove_book(book_path, rw_t)?;
  };
  Ok(())
}
