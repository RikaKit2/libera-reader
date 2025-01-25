use crate::db::{crud, models_impl::GetBookData, DB};
use crate::models::{Book, BookDataType, DataOfHashedBook, DataOfHashedBookKey, DataOfUnhashedBook};
use crate::models::{BookDataType::RepeatingSize, BookDataType::UniqueSize};
use crate::types::{BookPath, BookSize};
use crate::utils::{calc_file_hash, NotCachedBook};
use crate::vars::APP_DIRS;
use itertools::Itertools;
use native_db::ToInput;
use std::fs::remove_file;
use std::path::PathBuf;


pub(crate) fn get_all_from_db() -> Vec<Book> {
  let r_conn = DB.r_transaction().unwrap();
  r_conn.scan().primary().unwrap().all().unwrap().try_collect().unwrap()
}
pub(crate) fn get_num_of_books_of_this_size(book_size: BookSize) -> (usize, Option<DataOfUnhashedBook>) {
  let mut out_data: Option<DataOfUnhashedBook> = None;
  let mut num_of_book_with_this_size = 0;
  match crud::get_primary::<DataOfUnhashedBook>(book_size.clone()) {
    None => {
      let r_conn = DB.r_transaction().unwrap();
      for i in r_conn.scan().secondary::<DataOfHashedBook>(DataOfHashedBookKey::book_size).unwrap().all().unwrap() {
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
  (num_of_book_with_this_size, out_data)
}
pub(crate) fn update_book_data_type(book_path: BookPath, book_data_type: BookDataType) {
  let old_book = crud::get_primary::<Book>(book_path).unwrap();
  let mut new_book = old_book.clone();
  new_book.book_data_pk = book_data_type;
  crud::update(old_book, new_book).unwrap();
}

pub(crate) fn del_book_and_its_data(book: Book) {
  let book_data_type = book.book_data_pk.clone();
  match book_data_type {
    UniqueSize(book_size) => {
      let book_data = crud::get_primary::<DataOfUnhashedBook>(book_size).unwrap();
      delete_books_and_their_data(book_data, book);
    }
    RepeatingSize(book_hash) => {
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
    UniqueSize(book_size) => {
      remove_file(APP_DIRS.read().unwrap().dir_of_unhashed_books.join(book_size)).unwrap()
    }
    RepeatingSize(book_hash) => {
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

pub(crate) fn add_book(bookbuf: &PathBuf, book_size: BookSize) {
  let (
    db_book_count_with_this_size,
    data_of_unhashed_book
  ) = get_num_of_books_of_this_size(book_size.clone());

  if db_book_count_with_this_size == 0 {
    add_unique_size_book(bookbuf, book_size);
  } else if db_book_count_with_this_size == 1 {
    add_book_to_an_existing_one(bookbuf, book_size, data_of_unhashed_book.unwrap());
  } else if db_book_count_with_this_size > 1 {
    add_book_of_repeating_size(bookbuf, book_size);
  }
}
fn add_unique_size_book(bookbuf: &PathBuf, book_size: BookSize) {
  let book_path = bookbuf.to_str().unwrap().to_string();
  let book_data_type = UniqueSize(book_size.clone());
  crud::insert::<DataOfUnhashedBook>(DataOfUnhashedBook::new(book_size, vec![book_path.clone()])).unwrap();
  crud::insert::<Book>(Book::from_pathbuf(bookbuf, book_data_type.clone())).unwrap();
  NotCachedBook::new(book_path).push_to_storage();
}
fn add_book_to_an_existing_one(bookbuf: &PathBuf, book_size: BookSize, data_of_unhashed_book: DataOfUnhashedBook) {
  let path_of_other_book = &data_of_unhashed_book.book_data.books_pk[0];
  let path_of_new_book = bookbuf.to_str().unwrap().to_string();
  let hash_of_other_book = match &data_of_unhashed_book.book_hash {
    None => { calc_file_hash(bookbuf) }
    Some(hash_of_previus_book) => { hash_of_previus_book.clone() }
  };
  let hash_of_new_book = calc_file_hash(bookbuf);
  match hash_of_other_book.eq(&hash_of_new_book) {
    true => {
      update_book_data_type(path_of_other_book.clone(), RepeatingSize(hash_of_new_book.clone()));
      data_of_unhashed_book.replace_to_data_of_hashed_book(hash_of_new_book.clone());
    }
    false => {
      update_book_data_type(path_of_other_book.clone(), RepeatingSize(hash_of_other_book.clone()));
      data_of_unhashed_book.replace_to_data_of_hashed_book(hash_of_other_book);

      let new_book_data =
        DataOfHashedBook::new(hash_of_new_book.clone(), book_size, vec![path_of_new_book.clone()]);
      crud::insert::<DataOfHashedBook>(new_book_data).unwrap();
    }
  };
  let new_book = Book::from_pathbuf(bookbuf, RepeatingSize(hash_of_new_book));
  crud::insert::<Book>(new_book).unwrap();
  NotCachedBook::new(path_of_new_book).push_to_storage();
}
fn add_book_of_repeating_size(bookbuf: &PathBuf, book_size: BookSize) {
  let book_path = bookbuf.to_str().unwrap().to_string();
  let hash_of_new_book = calc_file_hash(bookbuf);
  match crud::get_primary::<DataOfHashedBook>(hash_of_new_book.clone()) {
    None => {
      let new_book_data = DataOfHashedBook::new(hash_of_new_book.clone(), book_size, vec![book_path.clone()]);
      crud::insert::<DataOfHashedBook>(new_book_data).unwrap();
      crud::insert::<Book>(Book::from_pathbuf(bookbuf, RepeatingSize(hash_of_new_book))).unwrap();
      NotCachedBook::new(book_path).push_to_storage();
    }
    Some(data_of_hashed_book) => {
      crud::insert::<Book>(Book::from_pathbuf(bookbuf, RepeatingSize(hash_of_new_book))).unwrap();
      match &data_of_hashed_book.book_data.cached {
        true => {}
        false => { NotCachedBook::new(book_path).push_to_storage(); }
      }
    }
  };
}

pub(crate) fn get_books_located_in_dir(path_to_dir: String) -> Vec<Book> {
  let r_conn = DB.r_transaction().unwrap();
  let books: Vec<Book> = r_conn.scan().primary().unwrap()
    .start_with(path_to_dir).unwrap().try_collect().unwrap();
  books
}
pub(crate) fn update_the_books_directory(old_dir_path: &PathBuf, new_dir_path: &PathBuf) {
  for old_book in get_books_located_in_dir(old_dir_path.to_str().unwrap().to_string()) {
    let mut new_book = old_book.clone();
    new_book.dir_name = new_dir_path.file_name().unwrap().to_str().unwrap().to_string();
    new_book.path_to_dir = new_dir_path.to_str().unwrap().to_string();
    new_book.path_to_book = new_dir_path.join(&old_book.book_name).to_str().unwrap().to_string();
    crud::update(old_book, new_book).unwrap();
  }
}
