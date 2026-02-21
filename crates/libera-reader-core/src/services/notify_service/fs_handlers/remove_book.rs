use anyhow::Ok;

use crate::db::{
  DB,
  models::books::{Books, book::BookPath},
};

pub(crate) async fn remove_book(book_path: BookPath, db: &DB) -> anyhow::Result<()> {
  if let Some(old_self) = Books::get_by_parent_dir(book_path.parent_dir.clone(), db)? {
    old_self.remove_book(book_path, db).await?;
  };
  Ok(())
}
