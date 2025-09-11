use std::vec;

use crate::db::{
  DB,
  models::{HashedBooks, UniqueBook},
};
use anyhow::{Ok, Result};

fn dir_deletion_handler_for_hashed_books(path_to_dir: &String, db: &DB) -> Result<()> {
  let hashed_books: Vec<HashedBooks> = db.scan_primary::<HashedBooks>()?;
  for i in hashed_books {
    let mut deleted_books = vec![];
    for (book_path, _) in i.books.iter() {
      if book_path.starts_with(path_to_dir) {
        deleted_books.push(book_path);
      }
    }
    if deleted_books.len() > 0 {
      if i.books.len() == deleted_books.len() {
        match i.user_data.can_delete() {
          true => {
            db.remove(i)?;
          }
          false => {
            i.mark_as_deleted(db);
          }
        }
      } else if deleted_books.len() < i.books.len() {
        let mut new_books_list = i.books.clone();
        for k in deleted_books {
          new_books_list.remove(k);
        }
        let mut new_hashed_books = i.clone();
        new_hashed_books.books = new_books_list;
        db.update(i, new_hashed_books)?;
      }
    }
  }
  Ok(())
}

pub(crate) fn dir_deletion_handler(path_to_dir: String, db: &DB) -> Result<()> {
  dir_deletion_handler_for_hashed_books(&path_to_dir, db)?;
  for i in UniqueBook::get_books_located_in_dir(path_to_dir, db)? {
    i.remove(db)?;
  }
  Ok(())
}
