use crate::db::crud;
use crate::db::models::{Book, DataOfHashedBook, DataOfUnhashedBook};
use crate::models::BookDataType::{RepeatingSize, UniqueSize};
use crate::types::{BookPath, BookSize};
use crate::utils::{calc_file_hash, calc_file_size_in_mb, NotCachedBook};
use crate::vars::TARGET_EXT;
use measure_time_macro::measure_time;
use std::path::PathBuf;
use tracing::debug;


#[measure_time]
pub(crate) fn book_adding_handler(book_pathbuf: &PathBuf) {
  let path_to_new_book = book_pathbuf.to_str().unwrap().to_string();
  let ext = book_pathbuf.extension().unwrap().to_str().unwrap().to_string();
  if TARGET_EXT.read().unwrap().contains(&ext) {
    let size_of_new_book = calc_file_size_in_mb(&path_to_new_book).to_string();
    let (db_book_count_with_this_size, data_of_unhashed_book) =
      Book::get_num_of_books_of_this_size(size_of_new_book.clone());

    if db_book_count_with_this_size == 0 {
      let book_data_type = UniqueSize(size_of_new_book.clone());
      NotCachedBook::new(book_data_type.clone(), path_to_new_book.clone()).push_to_storage();

      crud::insert::<DataOfUnhashedBook>(DataOfUnhashedBook::new(size_of_new_book, vec![path_to_new_book.clone()])).unwrap();
      crud::insert::<Book>(Book::from_pathbuf(book_pathbuf, book_data_type)).unwrap();
    } else if db_book_count_with_this_size == 1 {
      add_book_with_an_existing_size(data_of_unhashed_book.unwrap(), size_of_new_book, path_to_new_book, book_pathbuf);
    } else if db_book_count_with_this_size > 1 {
      let hash_of_new_book = calc_file_hash(book_pathbuf);
      match crud::get_primary::<DataOfHashedBook>(hash_of_new_book.clone()) {
        None => {
          NotCachedBook::new(RepeatingSize(hash_of_new_book.clone()), path_to_new_book.clone()).push_to_storage();
          let new_book_data =
            DataOfHashedBook::new(hash_of_new_book.clone(), size_of_new_book, vec![path_to_new_book]);
          crud::insert::<DataOfHashedBook>(new_book_data).unwrap();
        }
        Some(data_of_hashed_book) => {
          match &data_of_hashed_book.book_data.cached {
            true => {}
            false => {
              NotCachedBook::new(RepeatingSize(hash_of_new_book.clone()), path_to_new_book.clone()).push_to_storage();
            }
          }
        }
      };
      crud::insert::<Book>(Book::from_pathbuf(book_pathbuf, RepeatingSize(hash_of_new_book))).unwrap();
    }
  }
}

fn add_book_with_an_existing_size(data_of_unhashed_book: DataOfUnhashedBook,
                                  size_of_new_book: BookSize,
                                  path_to_new_book: BookPath, new_book_pathbuf: &PathBuf) {
  let path_of_other_book = &data_of_unhashed_book.book_data.books_pk[0];
  let hash_of_other_book = match &data_of_unhashed_book.book_hash {
    None => { calc_file_hash(new_book_pathbuf) }
    Some(hash_of_previus_book) => { hash_of_previus_book.clone() }
  };
  let hash_of_new_book = calc_file_hash(&new_book_pathbuf);
  match hash_of_other_book.eq(&hash_of_new_book) {
    true => {
      Book::update_book_data_type(path_of_other_book.clone(), RepeatingSize(hash_of_new_book.clone()));
      data_of_unhashed_book.replace_to_data_of_hashed_book(hash_of_new_book.clone());
    }
    false => {
      Book::update_book_data_type(path_of_other_book.clone(), RepeatingSize(hash_of_other_book.clone()));
      data_of_unhashed_book.replace_to_data_of_hashed_book(hash_of_other_book);

      let new_book_data = DataOfHashedBook::new(hash_of_new_book.clone(), size_of_new_book, vec![path_to_new_book]);
      crud::insert::<DataOfHashedBook>(new_book_data).unwrap();
    }
  };
  crud::insert::<Book>(Book::from_pathbuf(new_book_pathbuf, RepeatingSize(hash_of_new_book))).unwrap();
}
