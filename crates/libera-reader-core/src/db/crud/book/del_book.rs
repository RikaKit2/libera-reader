use crate::app_dirs::AppDirs;
use crate::db::models::{Book, BookDataWrapperPK, BookDataWrapperPK::RepeatingSize, BookDataWrapperPK::UniqueSize, DataOfHashedBook, DataOfUnhashedBook};
use crate::db::{crud, models_impl::GetBookData};
use crate::types::BookPath;
use native_db::{Database, ToInput};
use std::fs::remove_file;
use std::sync::{Arc, RwLock};

pub fn del_book(book: Book, app_dirs: &Arc<RwLock<AppDirs>>, db: &Database) {
  let book_data_type = book.book_data_wrapper_pk.clone();
  match book_data_type {
    UniqueSize(book_size) => {
      let book_data = crud::get_primary::<DataOfUnhashedBook>(book_size, db).unwrap();
      del_book_and_its_data(book_data, book, app_dirs, db);
    }
    RepeatingSize(book_hash) => {
      let book_data = crud::get_primary::<DataOfHashedBook>(book_hash, db).unwrap();
      del_book_and_its_data(book_data, book, app_dirs, db);
    }
  };
}
fn del_book_and_its_data<T: ToInput + GetBookData>(data: T, book: Book, app_dirs: &Arc<RwLock<AppDirs>>, db: &Database) {
  let rw_conn = db.rw_transaction().unwrap();
  let book_data = data.get_book_data_as_ref();
  if book_data.favorite == false && book_data.in_history == false {
    if book_data.cached {
      remove_thumbnail(&book.book_data_wrapper_pk, app_dirs);
    }
    if book_data.books_pk.len() == 1 {
      rw_conn.remove::<Book>(book).unwrap();
      rw_conn.remove::<T>(data).unwrap();
    } else if book_data.books_pk.len() > 1 {
      for i in book_data.books_pk.clone() {
        let book_for_deletion = crud::get_primary::<Book>(i, db).unwrap();
        rw_conn.remove::<Book>(book_for_deletion).unwrap();
      }
      rw_conn.remove::<T>(data).unwrap();
    }
  } else {
    mark_book_paths_as_invalid(book_data.books_pk.clone(), db);
  }
  rw_conn.commit().unwrap();
}
fn remove_thumbnail(book_data_type: &BookDataWrapperPK, app_dirs: &Arc<RwLock<AppDirs>>) {
  match book_data_type {
    UniqueSize(book_size) => remove_file(app_dirs.read().unwrap().inn.dir_of_unhashed_books.join(book_size)).unwrap(),
    RepeatingSize(book_hash) => remove_file(app_dirs.read().unwrap().inn.dir_of_hashed_books.join(book_hash)).unwrap(),
  };
}
fn mark_book_paths_as_invalid(books_pk: Vec<BookPath>, db: &Database) {
  books_pk.into_iter().for_each(|book_path| match crud::get_primary::<Book>(book_path, db) {
    None => {}
    Some(old_book) => {
      let mut new_book = old_book.clone();
      new_book.path_is_valid = false;
      crud::update::<Book>(old_book, new_book, db).unwrap()
    }
  });
}

