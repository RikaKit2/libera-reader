use crate::db::models::{Book, BookDataPK, DataOfHashedBook, DataOfUnhashedBook};
use crate::types::{BookSize, NotCachedBooks, DB};
use std::path::PathBuf;
use utils::calc_file_hash;

impl Book {
  pub(crate) fn insert_to_db(book_pathbuf: &PathBuf, book_size: BookSize, db: &DB, not_cached_books: &NotCachedBooks) {
    let mut books_with_such_size = Book::get_by_size(book_size, db).unwrap();
    let num_of_books_with_such_size = books_with_such_size.len();
    if num_of_books_with_such_size == 0 {
      Self::insert_to_books_of_unique_size(book_pathbuf, book_size, db, not_cached_books);
    } else if num_of_books_with_such_size == 1 {
      Self::replace_another_book_of_this_size(book_pathbuf, books_with_such_size.remove(0), book_size, db, not_cached_books);
    } else {
      Self::insert_to_hashed_books(book_pathbuf, book_size, db, not_cached_books);
    }
  }
  pub(crate) fn insert_to_books_of_unique_size(book_pathbuf: &PathBuf, book_size: BookSize, db: &DB, not_cached_books: &NotCachedBooks) {
    let new_book = Book::from_pathbuf(book_pathbuf, BookDataPK::UniqueSize(book_size.clone()));
    let book_data = DataOfUnhashedBook::new(book_size, vec![new_book.full_path.clone()]);
    db.insert::<Book>(new_book.clone()).unwrap();
    db.insert::<DataOfUnhashedBook>(book_data).unwrap();
    not_cached_books.push(Box::new(new_book)).unwrap();
    
  }
  fn replace_another_book_of_this_size(book_pathbuf: &PathBuf, other_book: Book, book_size: BookSize, db: &DB, not_cached_books: &NotCachedBooks) {
    let data_of_book_with_such_size = db.get_primary::<DataOfUnhashedBook>(book_size).unwrap().unwrap();

    let hash_of_updated_book = calc_file_hash(PathBuf::from(&other_book.full_path)).unwrap();
    let hash_of_new_book = calc_file_hash(book_pathbuf).unwrap();

    let new_book = Book::from_pathbuf(book_pathbuf, BookDataPK::Hashed(hash_of_new_book.clone()));

    let mut updated_book = other_book.clone();
    let full_path_to_updated_book = updated_book.full_path.clone();
    updated_book.book_data_pk = BookDataPK::Hashed(hash_of_updated_book.clone());
    db.update::<Book>(other_book.clone(), updated_book).unwrap();

    db.remove::<DataOfUnhashedBook>(data_of_book_with_such_size.clone()).unwrap();
    match hash_of_new_book.eq(&hash_of_updated_book) {
      true => {
        let mut old_book_data = data_of_book_with_such_size.book_data;
        old_book_data.books_pk.clear();
        old_book_data.books_pk.extend(vec![new_book.full_path.clone(), full_path_to_updated_book.clone()]);

        db.insert::<Book>(new_book.clone()).unwrap();
        let other_book_is_cached = old_book_data.cached;
        let new_book_data = DataOfHashedBook::new_with_other_book_data(hash_of_updated_book, book_size, old_book_data);
        db.insert::<DataOfHashedBook>(new_book_data).unwrap();
        match other_book_is_cached {
          true => {}
          false => {
            not_cached_books.push(Box::new(new_book)).unwrap();
          }
        };
      }
      false => {
        let book_data_of_updated_book = DataOfHashedBook::new(hash_of_updated_book, book_size.clone(),
                                                              vec![full_path_to_updated_book]);
        let book_data_of_new_book = DataOfHashedBook::new(hash_of_new_book, book_size, vec![new_book.full_path.clone()]);

        db.insert::<DataOfHashedBook>(book_data_of_updated_book).unwrap();
        db.insert::<DataOfHashedBook>(book_data_of_new_book).unwrap();
        db.insert::<Book>(new_book.clone()).unwrap();
        not_cached_books.push(Box::new(new_book)).unwrap();
        not_cached_books.push(Box::new(other_book.clone())).unwrap();
      }
    }
    
  }
  pub(crate) fn insert_to_hashed_books(book_pathbuf: &PathBuf, book_size: BookSize, db: &DB, not_cached_books: &NotCachedBooks) {
    let hash_of_new_book = calc_file_hash(&book_pathbuf).unwrap();
    let new_book = Book::from_pathbuf(book_pathbuf, BookDataPK::Hashed(hash_of_new_book.clone()));
    match db.get_primary::<DataOfHashedBook>(hash_of_new_book.clone()).unwrap() {
      None => {
        let new_book_data = DataOfHashedBook::new(hash_of_new_book, book_size, vec![new_book.full_path.clone()]);
        db.insert::<DataOfHashedBook>(new_book_data).unwrap();
        db.insert::<Book>(new_book.clone()).unwrap();
        not_cached_books.push(Box::new(new_book)).unwrap();
      }
      Some(other_book_data) => {
        match other_book_data.book_data.cached {
          true => {
            db.insert::<Book>(new_book.clone()).unwrap();
            not_cached_books.push(Box::new(new_book)).unwrap();
          }
          false => {}
        }
      }
    };
  }
}
