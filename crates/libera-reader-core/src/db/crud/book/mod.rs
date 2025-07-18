mod del_book;
mod add_book;
use crate::db::crud;
use crate::db::models::data_of_hashed_book::DataOfHashedBookKey;
use crate::db::models::{Book, BookDataWrapperPK, DataOfHashedBook, DataOfUnhashedBook};
use crate::types::{BookPath, BookSize, DB};
pub use add_book::add_book;
use anyhow::Result;
pub use del_book::del_book;
use itertools::Itertools;
use std::path::PathBuf;

pub(crate) fn get_num_of_books_of_this_size(book_size: BookSize, db: &DB) -> Result<(usize, Option<DataOfUnhashedBook>)> {
  let mut out_data: Option<DataOfUnhashedBook> = None;
  let mut num_of_book_with_this_size = 0;
  match crud::get_primary::<DataOfUnhashedBook>(book_size.clone(), db)? {
    None => {
      let r_conn = db.r_transaction()?;
      for i in r_conn.scan().secondary::<DataOfHashedBook>(DataOfHashedBookKey::book_size)?.all()? {
        match i {
          Ok(_data) => { num_of_book_with_this_size += 1; }
          Err(_) => {}
        }
      }
    }
    Some(data) => {
      num_of_book_with_this_size = data.book_data.books_pk.len();
      out_data = Some(data);
    }
  };
  Ok((num_of_book_with_this_size, out_data))
}
pub(crate) fn update_book_data_type(book_path: BookPath, book_data_type: BookDataWrapperPK, db: &DB) -> Result<()> {
  let old_book = crud::get_primary::<Book>(book_path, db)?.unwrap();
  let mut new_book = old_book.clone();
  new_book.book_data_wrapper_pk = book_data_type;
  crud::update(old_book, new_book, db)?;
  Ok(())
}
pub(crate) fn get_books_located_in_dir(path_to_dir: String, db: &DB) -> Result<Vec<Book>> {
  let r_conn = db.r_transaction()?;
  let books: Vec<Book> = r_conn.scan().primary().unwrap().start_with(path_to_dir)?.try_collect()?;
  Ok(books)
}
pub(crate) fn update_the_books_directory(old_dir_path: &PathBuf, new_dir_path: &PathBuf, db: &DB) -> Result<()> {
  for old_book in get_books_located_in_dir(old_dir_path.to_str().unwrap().to_string(), db)? {
    let mut new_book = old_book.clone();
    new_book.dir_name = new_dir_path.file_name().unwrap().to_str().unwrap().to_string();
    new_book.path_to_dir = new_dir_path.to_str().unwrap().to_string();
    new_book.path_to_book = new_dir_path.join(&old_book.book_name).to_str().unwrap().to_string();
    crud::update(old_book, new_book, db)?
  }
  Ok(())
}
