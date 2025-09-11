use crate::db::{
  DB,
  models::{BookFsData, HashedBooks, UniqueBook},
};
use anyhow::Result;
use std::path::PathBuf;

pub(crate) async fn book_path_update_handler(old_path: &PathBuf, new_path: &PathBuf, db: &DB) -> Result<()> {
  let old_path_str = old_path.to_str().unwrap().to_string();
  let new_path_str = new_path.to_str().unwrap().to_string();
  match HashedBooks::get_by_path(old_path_str.clone(), db)? {
    Some(old_book) => {
      let mut new_book = old_book.clone();
      new_book.books.remove(&old_path_str).unwrap();
      new_book.books.insert(new_path_str, BookFsData::from_pathbuf(new_path));
      db.update(old_book, new_book);
    }
    None => match UniqueBook::get_by_path(old_path_str.clone(), db)? {
      Some(old_book) => {
        let mut new_book = old_book.clone();
        new_book.full_path = new_path_str;
        new_book.fs_data = BookFsData::from_pathbuf(new_path);
        db.update(old_book, new_book);
      }
      None => {}
    },
  };
  Ok(())
}
