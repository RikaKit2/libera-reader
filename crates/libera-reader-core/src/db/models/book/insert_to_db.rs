use crate::db::crud::{get_primary, insert, remove, update};
use crate::db::models::{Book, BookDataPK, DataOfHashedBook, DataOfUnhashedBook};
use crate::types::{BookSize, NotCachedBooks, DB};
use anyhow::Result;
use std::path::PathBuf;
use utils::calc_file_hash;

impl Book {
  pub(crate) fn insert_to_db(book_pathbuf: &PathBuf, book_size: BookSize, db: &DB, not_cached_books: &NotCachedBooks) -> Result<()> {
    let mut books_with_such_size = Book::get_by_size(book_size, db)?;
    let num_of_books_with_such_size = books_with_such_size.len();
    if num_of_books_with_such_size == 0 {
      Self::insert_to_unique_size_books(book_pathbuf, book_size, db)?;
    } else if num_of_books_with_such_size == 1 {
      Self::replace_another_book_of_this_size(book_pathbuf, books_with_such_size.remove(0), book_size, db, not_cached_books)?;
    } else {
      Self::insert_to_hashed_books(book_pathbuf, book_size, db, not_cached_books)?;
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
  fn replace_another_book_of_this_size(book_pathbuf: &PathBuf, other_book: Book, book_size: BookSize, db: &DB, not_cached_books: &NotCachedBooks) -> Result<()> {
    let data_of_book_with_such_size = get_primary::<DataOfUnhashedBook>(book_size, db)?.unwrap();
    let mut updated_book = other_book.clone();

    let hash_of_updated_book = calc_file_hash(PathBuf::from(&other_book.full_path))?;
    let hash_of_new_book = calc_file_hash(book_pathbuf)?;

    let new_book = Book::from_pathbuf(book_pathbuf, BookDataPK::RepeatingSize(hash_of_new_book.clone()));

    let full_path_to_updated_book = updated_book.full_path.clone();

    updated_book.book_data_pk = BookDataPK::RepeatingSize(hash_of_updated_book.clone());
    update::<Book>(other_book.clone(), updated_book, db)?;

    remove::<DataOfUnhashedBook>(data_of_book_with_such_size.clone(), db)?;
    match hash_of_new_book.eq(&hash_of_updated_book) {
      true => {
        let mut old_book_data = data_of_book_with_such_size.book_data;
        old_book_data.books_pk.clear();
        old_book_data.books_pk.extend(vec![new_book.full_path.clone(), full_path_to_updated_book.clone()]);

        insert::<Book>(new_book.clone(), db)?;
        match old_book_data.thumbnail.is_some() {
          true => {}
          false => {
            not_cached_books.push(Box::new(new_book))?;
          }
        };
        let new_book_data = DataOfHashedBook::new_with_other_book_data(hash_of_updated_book, book_size, old_book_data);
        insert::<DataOfHashedBook>(new_book_data, db)?;
      }
      false => {
        let book_data_of_updated_book = DataOfHashedBook::new(hash_of_updated_book, book_size.clone(),
                                                              vec![full_path_to_updated_book]);
        let book_data_of_new_book = DataOfHashedBook::new(hash_of_new_book, book_size, vec![new_book.full_path.clone()]);

        insert::<DataOfHashedBook>(book_data_of_updated_book, db)?;
        insert::<DataOfHashedBook>(book_data_of_new_book, db)?;
        insert::<Book>(new_book.clone(), db)?;
        not_cached_books.push(Box::new(new_book))?;
        not_cached_books.push(Box::new(other_book.clone()))?;
      }
    }
    Ok(())
  }
  fn insert_to_hashed_books(book_pathbuf: &PathBuf, book_size: BookSize, db: &DB, not_cached_books: &NotCachedBooks) -> Result<()> {
    let hash_of_new_book = calc_file_hash(&book_pathbuf)?;
    let new_book = Book::from_pathbuf(book_pathbuf, BookDataPK::RepeatingSize(hash_of_new_book.clone()));
    match get_primary::<DataOfHashedBook>(hash_of_new_book.clone(), db)? {
      None => {
        let new_book_data = DataOfHashedBook::new(hash_of_new_book, book_size, vec![new_book.full_path.clone()]);
        insert::<DataOfHashedBook>(new_book_data, db)?;
        insert::<Book>(new_book.clone(), db)?;
        not_cached_books.push(Box::new(new_book))?;
      }
      Some(other_book_data) => {
        match other_book_data.book_data.thumbnail.is_none() {
          true => {
            insert::<Book>(new_book.clone(), db)?;
            not_cached_books.push(Box::new(new_book))?;
          }
          false => {}
        }
      }
    };
    Ok(())
  }
}
