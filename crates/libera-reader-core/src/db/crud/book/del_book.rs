use crate::db::crud;
use crate::db::models::{Book, BookDataWrapperPK, BookDataWrapperPK::RepeatingSize,
                        BookDataWrapperPK::UniqueSize, DataOfHashedBook, DataOfUnhashedBook, GetBookData};
use crate::types::{BookPath, APP_DIRS, DB};
use anyhow::Result;
use native_db::ToInput;
use std::fs::remove_file;

pub fn del_book(book: Book, app_dirs: &APP_DIRS, db: &DB) -> Result<()> {
  let book_data_type = book.book_data_wrapper_pk.clone();
  match book_data_type {
    UniqueSize(book_size) => {
      let book_data = crud::get_primary::<DataOfUnhashedBook>(book_size, db)?.unwrap();
      del_book_and_its_data(book_data, book, app_dirs, db)?;
    }
    RepeatingSize(book_hash) => {
      let book_data = crud::get_primary::<DataOfHashedBook>(book_hash, db)?.unwrap();
      del_book_and_its_data(book_data, book, app_dirs, db)?;
    }
  };
  Ok(())
}
fn del_book_and_its_data<T: ToInput + GetBookData>(data: T, book: Book, app_dirs: &APP_DIRS, db: &DB) -> Result<()> {
  let rw_conn = db.rw_transaction()?;
  let book_data = data.get_book_data_as_ref();
  if book_data.favorite == false && book_data.in_history == false {
    if book_data.cached {
      remove_thumbnail(&book.book_data_wrapper_pk, app_dirs);
    }
    if book_data.books_pk.len() == 1 {
      rw_conn.remove::<Book>(book)?;
      rw_conn.remove::<T>(data)?;
    } else if book_data.books_pk.len() > 1 {
      for i in book_data.books_pk.clone() {
        let book_for_deletion = crud::get_primary::<Book>(i, db)?.unwrap();
        rw_conn.remove::<Book>(book_for_deletion)?;
      }
      rw_conn.remove::<T>(data)?;
    }
  } else {
    mark_book_paths_as_invalid(book_data.books_pk.clone(), db)?;
  }
  rw_conn.commit()?;
  Ok(())
}
fn remove_thumbnail(book_data_type: &BookDataWrapperPK, app_dirs: &APP_DIRS) {
  match book_data_type {
    UniqueSize(book_size) => remove_file(app_dirs.read().unwrap().inn.dir_of_unhashed_books.join(book_size)).unwrap(),
    RepeatingSize(book_hash) => remove_file(app_dirs.read().unwrap().inn.dir_of_hashed_books.join(book_hash)).unwrap(),
  };
}
fn mark_book_path_as_invalid(book_path: BookPath, db: &DB) -> Result<()> {
  let book = crud::get_primary::<Book>(book_path, db)?;
  match book {
    None => {}
    Some(old_book) => {
      let mut new_book = old_book.clone();
      new_book.path_is_valid = false;
      crud::update::<Book>(old_book, new_book, db)?
    }
  }
  Ok(())
}
fn mark_book_paths_as_invalid(books_pk: Vec<BookPath>, db: &DB) -> Result<()> {
  for book_path in books_pk {
    mark_book_path_as_invalid(book_path, db)?
  }
  Ok(())
}

