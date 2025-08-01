use crate::db::crud::{get_primary, insert, remove, update};
use crate::db::models::{Book, BookDataPK, DataOfHashedBook, DataOfUnhashedBook};
use crate::types::{BookSize, DB};
use anyhow::Result;
use std::path::PathBuf;
use utils::calc_file_hash;

impl Book {
  pub(crate) fn insert_to_db(book_pathbuf: &PathBuf, book_size: BookSize, db: &DB) -> Result<()> {
    match get_primary::<DataOfUnhashedBook>(book_size.clone(), db)? {
      None => {
        // either the book is new, or there are books with its size in hashed books
        // check for the presence of books with the same size in hashed books
        // if there are no books with such sizes in hashed books, then add it to unique books
        let hashed_books_with_such_size: Vec<DataOfHashedBook> = DataOfHashedBook::find_by_size(book_size.clone(), db)?;
        if hashed_books_with_such_size.len() == 0 {
          Self::insert_to_unique_size_books(book_pathbuf, book_size, db)?;
        } else {
          let hash_of_new_book = calc_file_hash(book_pathbuf)?;
          let new_book = Book::from_pathbuf(book_pathbuf, BookDataPK::RepeatingSize(hash_of_new_book.clone()));
          let book_with_same_size: &DataOfHashedBook =
            hashed_books_with_such_size.iter().find(|i| i.book_size.eq(&book_size)).unwrap();
          match book_with_same_size.book_hash.eq(&hash_of_new_book) {
            true => {}
            false => {
              let new_book_data = DataOfUnhashedBook::new(book_size, vec![new_book.full_path.clone()]);
              insert::<DataOfUnhashedBook>(new_book_data, db)?;
            }
          };
          insert::<Book>(new_book, db)?;
        }
      }
      Some(data_of_book_with_such_size) => {
        let book_with_such_size = get_primary::<Book>(data_of_book_with_such_size.book_data.books_pk[0].clone(), db)?.unwrap();
        let mut updated_book = book_with_such_size.clone();

        let hash_of_updated_book = calc_file_hash(PathBuf::from(&book_with_such_size.full_path))?;
        let hash_of_new_book = calc_file_hash(book_pathbuf)?;

        let new_book = Book::from_pathbuf(book_pathbuf, BookDataPK::RepeatingSize(hash_of_new_book.clone()));

        let full_path_to_updated_book = updated_book.full_path.clone();

        updated_book.book_data_pk = BookDataPK::RepeatingSize(hash_of_updated_book.clone());
        update::<Book>(book_with_such_size, updated_book, db)?;

        remove::<DataOfUnhashedBook>(data_of_book_with_such_size.clone(), db)?;
        match hash_of_new_book.eq(&hash_of_updated_book) {
          true => {
            let mut old_book_data =  data_of_book_with_such_size.book_data;
            old_book_data.books_pk.clear();
            old_book_data.books_pk.extend(vec![new_book.full_path.clone(), full_path_to_updated_book.clone()]);
            let new_book_data = DataOfHashedBook::new_with_other_book_data(hash_of_updated_book, book_size, old_book_data);
            insert::<DataOfHashedBook>(new_book_data, db)?;
          }
          false => {
            let book_data_of_updated_book = DataOfHashedBook::new(hash_of_updated_book, book_size.clone(),
                                                                  vec![full_path_to_updated_book]);
            let book_data_of_new_book = DataOfHashedBook::new(hash_of_new_book, book_size, vec![new_book.full_path.clone()]);
            insert::<DataOfHashedBook>(book_data_of_updated_book, db)?;
            insert::<DataOfHashedBook>(book_data_of_new_book, db)?;
          }
        }
        insert::<Book>(new_book, db)?;
      }
    }
    Ok(())
  }
  fn insert_to_unique_size_books(book_pathbuf: &PathBuf, book_size: BookSize, db: &DB) -> Result<()> {
    let new_book = Book::from_pathbuf(book_pathbuf, BookDataPK::UniqueSize(book_size.clone()));
    let book_data = DataOfUnhashedBook::new(book_size, vec![new_book.full_path.clone()]);
    insert::<Book>(new_book, db)?;
    insert::<DataOfUnhashedBook>(book_data, db)?;
    Ok(())
  }
}
