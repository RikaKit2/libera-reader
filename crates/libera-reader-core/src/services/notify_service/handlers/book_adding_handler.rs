use crate::db::crud;
use crate::db::models::{Book, BookDataType, DataOfHashedBook, DataOfUnhashedBook};
use crate::types::NotCachedBook;
use crate::utils::{calc_file_hash, calc_file_size_in_mb};
use crate::vars::{NOT_CACHED_BOOKS, TARGET_EXT};
use measure_time_macro::measure_time;
use std::path::PathBuf;
use tracing::debug;


#[measure_time]
pub(crate) fn book_adding_handler(book_pathbuf: &PathBuf) {
  let path_to_book = book_pathbuf.to_str().unwrap().to_string();
  let ext = book_pathbuf.extension().unwrap().to_str().unwrap().to_string();
  if TARGET_EXT.read().unwrap().contains(&ext) {
    let book_size = calc_file_size_in_mb(&path_to_book).to_string();
    match crud::get_primary::<DataOfUnhashedBook>(book_size.clone()) {
      None => {
        let book_data_type = BookDataType::UniqueSize(book_size.clone());

        NOT_CACHED_BOOKS.push(NotCachedBook { data_type: book_data_type.clone(), path_to_book: path_to_book.clone() });

        crud::insert::<DataOfUnhashedBook>(DataOfUnhashedBook::new(book_size, vec![path_to_book.clone()])).unwrap();

        crud::insert::<Book>(Book::from_pathbuf(&book_pathbuf, book_data_type)).unwrap();
      }
      Some(unique_book_data) => {
        let book_hash = match &unique_book_data.book_hash {
          None => {
            calc_file_hash(&path_to_book)
          }
          Some(book_hash) => {
            book_hash.clone()
          }
        };
        let book_path = &unique_book_data.book_data.books_pk[0];
        let book_from_db = crud::get_primary::<Book>(book_path.clone()).unwrap();

        let new_book = Book::from_pathbuf(&book_pathbuf, BookDataType::RepeatingSize(book_hash.clone()));
        crud::update(book_from_db, new_book).unwrap();

        let new_book_data = crud::remove::<DataOfUnhashedBook>(unique_book_data).unwrap().to_data_of_hashed_book(book_hash);
        crud::insert::<DataOfHashedBook>(new_book_data).unwrap();
      }
    };
  }
}
