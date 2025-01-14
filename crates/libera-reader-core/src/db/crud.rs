use crate::db::DB;
use native_db::{db_type, ToInput, ToKey};


pub fn get_primary<T: ToInput>(key: impl ToKey) -> Option<T> {
  let r_conn = DB.r_transaction().unwrap();
  r_conn.get().primary(key).unwrap()
}

//noinspection RsUnwrap
pub fn insert<T: ToInput>(item: T) -> db_type::Result<()> {
  let rw_conn = DB.rw_transaction().unwrap();
  rw_conn.insert(item).unwrap();
  rw_conn.commit()
}

//noinspection RsUnwrap
pub fn update<T: ToInput>(old_data: T, new_data: T) -> db_type::Result<()> {
  let rw_conn = DB.rw_transaction().unwrap();
  rw_conn.update(old_data, new_data).unwrap();
  rw_conn.commit()
}

//noinspection RsUnwrap
pub fn remove<T: ToInput>(item: T) -> Result<T, db_type::Error> {
  let rw_conn = DB.rw_transaction().unwrap();
  let res = rw_conn.remove(item).unwrap();
  match rw_conn.commit() {
    Ok(_) => {
      Ok(res)
    }
    Err(e) => {
      Err(e)
    }
  }
}

pub(crate) mod book {
  use crate::db::models_impl::GetBookData;
  use crate::db::{crud, DB};
  use crate::models::{Book, BookDataType, DataOfHashedBook, DataOfUnhashedBook};
  use crate::types::BookPath;
  use crate::vars::APP_DIRS;
  use native_db::ToInput;
  use std::fs::remove_file;

  pub(crate) fn del_book_and_its_data(book: Book) {
    let book_data_type = book.book_data_pk.clone();
    match book_data_type {
      BookDataType::UniqueSize(book_size) => {
        let book_data = crud::get_primary::<DataOfUnhashedBook>(book_size).unwrap();
        delete_books_and_their_data(book_data, book);
      }
      BookDataType::RepeatingSize(book_hash) => {
        let book_data = crud::get_primary::<DataOfHashedBook>(book_hash).unwrap();
        delete_books_and_their_data(book_data, book);
      }
    };
  }

  fn delete_books_and_their_data<T: ToInput + GetBookData>(data: T, book: Book) {
    let rw_conn = DB.rw_transaction().unwrap();
    let book_data = data.get_book_data_as_ref();
    if book_data.favorite == false && book_data.in_history == false {
      if book_data.cached {
        remove_thumbnail(&book.book_data_pk);
      }
      if book_data.books_pk.len() == 1 {
        rw_conn.remove::<Book>(book).unwrap();
        rw_conn.remove::<T>(data).unwrap();
      } else if book_data.books_pk.len() > 1 {
        for i in book_data.books_pk.clone() {
          let book_for_deletion = crud::get_primary::<Book>(i).unwrap();
          rw_conn.remove::<Book>(book_for_deletion).unwrap();
        }
        rw_conn.remove::<T>(data).unwrap();
      }
    } else {
      mark_book_paths_as_invalid(book_data.books_pk.clone());
    }
    rw_conn.commit().unwrap();
  }
  fn remove_thumbnail(book_data_type: &BookDataType) {
    match book_data_type {
      BookDataType::UniqueSize(book_size) => {
        remove_file(APP_DIRS.read().unwrap().dir_of_unhashed_books.join(book_size)).unwrap()
      }
      BookDataType::RepeatingSize(book_hash) => {
        remove_file(APP_DIRS.read().unwrap().dir_of_hashed_books.join(book_hash)).unwrap()
      }
    };
  }
  fn mark_book_paths_as_invalid(books_pk: Vec<BookPath>) {
    books_pk.into_iter().for_each(|book_path| {
      match crud::get_primary::<Book>(book_path) {
        None => {}
        Some(old_book) => {
          let mut new_book = old_book.clone();
          new_book.path_is_valid = false;
          crud::update::<Book>(old_book, new_book).unwrap()
        }
      }
    });
  }
}
