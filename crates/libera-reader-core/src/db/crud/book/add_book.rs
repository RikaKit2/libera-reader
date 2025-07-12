use crate::db::crud;
use crate::db::crud::book::{get_num_of_books_of_this_size, update_book_data_type};
use crate::db::models::{Book, BookDataWrapperPK::RepeatingSize, BookDataWrapperPK::UniqueSize, DataOfHashedBook, DataOfUnhashedBook};
use crate::types::{BookSize, NotCachedBooks, DB};
use anyhow::Result;
use std::path::PathBuf;
use utils::calc_file_hash;

pub fn add_book(book_pathbuf: &PathBuf, book_size: BookSize, db: &DB, not_cached_books: &NotCachedBooks) -> Result<()> {
  let (db_book_count_with_this_size, data_of_unhashed_book) = get_num_of_books_of_this_size(book_size.clone(), db)?;

  if db_book_count_with_this_size == 0 {
    add_unhashed_book(book_pathbuf, book_size, db, not_cached_books)?;
  } else if db_book_count_with_this_size == 1 {
    replace_unhashed_book_with_hashed(book_pathbuf, book_size, data_of_unhashed_book.unwrap(), db, not_cached_books)?;
  } else if db_book_count_with_this_size > 1 {
    add_hashed_book(book_pathbuf, book_size, db, not_cached_books)?;
  }
  Ok(())
}
fn add_unhashed_book(book_pathbuf: &PathBuf, book_size: BookSize, db: &DB, not_cached_books: &NotCachedBooks) -> Result<()> {
  let book_path = book_pathbuf.to_str().unwrap().to_string();
  let book_data_type = UniqueSize(book_size.clone());
  let new_book = Book::from_pathbuf(book_pathbuf, book_data_type.clone());
  crud::insert::<DataOfUnhashedBook>(DataOfUnhashedBook::new(book_size, vec![book_path]), db)?;
  crud::insert::<Book>(new_book.clone(), db)?;
  not_cached_books.push(Box::new(new_book))?;
  Ok(())
}
fn add_hashed_book(book_pathbuf: &PathBuf, book_size: BookSize, db: &DB, not_cached_books: &NotCachedBooks) -> Result<()> {
  let hash_of_new_book = calc_file_hash(book_pathbuf)?;
  let new_book = Book::from_pathbuf(book_pathbuf, RepeatingSize(hash_of_new_book.clone()));
  match crud::get_primary::<DataOfHashedBook>(hash_of_new_book.clone(), db)? {
    None => {
      let book_path = book_pathbuf.to_str().unwrap().to_string();
      let new_book_data = DataOfHashedBook::new(hash_of_new_book, book_size, vec![book_path]);
      crud::insert::<DataOfHashedBook>(new_book_data, db)?;
      not_cached_books.push(Box::new(new_book))?;
    }
    Some(data_of_hashed_book) => {
      crud::insert::<Book>(new_book.clone(), db)?;
      match &data_of_hashed_book.book_data.cached {
        true => {}
        false => { not_cached_books.push(Box::new(new_book))?; }
      }
    }
  };
  Ok(())
}
fn replace_unhashed_book_with_hashed(book_pathbuf: &PathBuf, book_size: BookSize,
                                     data_of_unhashed_book: DataOfUnhashedBook,
                                     db: &DB, not_cached_books: &NotCachedBooks) -> Result<()> {
  let path_of_other_book = &data_of_unhashed_book.book_data.books_pk[0];
  let path_of_new_book = book_pathbuf.to_str().unwrap().to_string();
  let hash_of_other_book = match &data_of_unhashed_book.book_hash {
    None => calc_file_hash(book_pathbuf)?,
    Some(hash_of_previous_book) => hash_of_previous_book.clone(),
  };
  let hash_of_new_book = calc_file_hash(book_pathbuf)?;
  match hash_of_other_book.eq(&hash_of_new_book) {
    true => {
      update_book_data_type(path_of_other_book.clone(), RepeatingSize(hash_of_new_book.clone()), db)?;
      data_of_unhashed_book.replace_to_data_of_hashed_book(hash_of_new_book.clone(), db);
    }
    false => {
      update_book_data_type(path_of_other_book.clone(), RepeatingSize(hash_of_other_book.clone()), db)?;
      data_of_unhashed_book.replace_to_data_of_hashed_book(hash_of_other_book, db);

      let new_book_data = DataOfHashedBook::new(hash_of_new_book.clone(), book_size, vec![path_of_new_book]);
      crud::insert::<DataOfHashedBook>(new_book_data, db)?;
    }
  };
  let new_book = Book::from_pathbuf(book_pathbuf, RepeatingSize(hash_of_new_book));
  crud::insert::<Book>(new_book.clone(), db)?;
  not_cached_books.push(Box::new(new_book))?;
  Ok(())
}
