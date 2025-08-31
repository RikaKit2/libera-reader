use crate::db::DB;
use crate::db::models::{Book, BookDataPK, DataOfHashedBook, DataOfUnhashedBook};
use crate::types::{BookHash, BookSize, NotCachedBooks};
use std::path::PathBuf;
use std::vec;
use utils::calc_file_hash;

impl Book {
  pub(crate) async fn insert(book_pathbuf: &PathBuf, book_size: BookSize, db: &DB, not_cached_books: &NotCachedBooks) -> anyhow::Result<()> {
    match db.get_primary::<DataOfUnhashedBook>(book_size.clone())? {
      Some(key_data) => {
        Self::move_unique_books_to_hashed(book_pathbuf, key_data, book_size, db, not_cached_books).await;
      }
      None => {
        let num_of_hashed_books = DataOfHashedBook::scan_by_size(book_size, db)?.len();
        if num_of_hashed_books > 0 {
          let book_hash: BookHash = calc_file_hash(book_pathbuf).await.unwrap();
          Self::insert_book_to_hashed(book_pathbuf, book_size, book_hash, db, not_cached_books).expect("Failed to insert hashed books");
        } else {
          Self::insert_book_of_unique_size(book_pathbuf, book_size, db, not_cached_books);
        }
      }
    };
    Ok(())
  }
  pub(crate) fn insert_book_of_unique_size(book_pathbuf: &PathBuf, book_size: BookSize, db: &DB, not_cached_books: &NotCachedBooks) {
    let new_book = Book::from_pathbuf(book_pathbuf, BookDataPK::UniqueSize(book_size.clone()));
    let book_data = DataOfUnhashedBook::new(book_size, vec![new_book.full_path.clone()]);
    db.insert::<DataOfUnhashedBook>(book_data).unwrap();
    db.insert::<Book>(new_book).unwrap();
    not_cached_books.push(Box::new(book_pathbuf.clone())).unwrap();
  }
  pub(crate) async fn move_unique_books_to_hashed(
    new_pathbuf: &PathBuf, key_data: DataOfUnhashedBook, book_size: BookSize, db: &DB, not_cached_books: &NotCachedBooks,
  ) {
    let previous_book = Book::get_by_path(&key_data.book_data.books_pk[0], db).unwrap().unwrap();
    let previous_book_pathbuf = previous_book.to_pathbuf();
    let previous_book_is_cached = key_data.book_data.cached.clone();

    let hash_of_previous_book = calc_file_hash(PathBuf::from(&previous_book.full_path)).await.unwrap();
    let hash_of_new_book = calc_file_hash(new_pathbuf).await.unwrap();
    let new_book = Book::from_pathbuf(new_pathbuf, BookDataPK::Hashed(hash_of_new_book.clone()));

    let mut key_data_of_previous_book = DataOfHashedBook::new_with_book_data(hash_of_previous_book.clone(), book_size, key_data.book_data.clone());
    key_data_of_previous_book.book_data.books_pk.push(new_book.full_path.clone());
    db.insert(key_data_of_previous_book).unwrap();

    match hash_of_previous_book.eq(&hash_of_new_book) {
      true => {
        let mut modifyed_previous_book = previous_book.clone();
        modifyed_previous_book.book_data_pk = BookDataPK::Hashed(hash_of_previous_book.clone());
        db.update(previous_book, modifyed_previous_book).unwrap();
        db.insert(new_book).unwrap();
      }
      false => {
        let key_data_of_new_book = DataOfHashedBook::new(hash_of_new_book.clone(), book_size, vec![new_book.full_path.clone()]);
        db.insert(key_data_of_new_book).unwrap();
        db.insert(new_book).unwrap();

        not_cached_books.push(Box::new(new_pathbuf.clone())).unwrap();
      }
    }
    db.remove(key_data).unwrap();

    if previous_book_is_cached == false {
      not_cached_books.push(Box::new(previous_book_pathbuf)).unwrap();
    }
  }
  pub(crate) fn insert_book_to_hashed(
    book_pathbuf: &PathBuf, book_size: BookSize, book_hash: BookHash, db: &DB, not_cached_books: &NotCachedBooks,
  ) -> anyhow::Result<()> {
    let new_book = Book::from_pathbuf(book_pathbuf, BookDataPK::Hashed(book_hash.clone()));
    match db.get_primary::<DataOfHashedBook>(book_hash.clone())? {
      None => {
        let new_book_data = DataOfHashedBook::new(book_hash, book_size, vec![new_book.full_path.clone()]);
        db.insert::<DataOfHashedBook>(new_book_data)?;
        db.insert::<Book>(new_book)?;
        not_cached_books.push(Box::new(book_pathbuf.clone()))?;
      }
      Some(other_book_data) => match other_book_data.book_data.cached {
        true => {
          db.insert::<Book>(new_book)?;
          not_cached_books.push(Box::new(book_pathbuf.clone()))?;
        }
        false => {}
      },
    };
    Ok(())
  }
}
