use std::path::PathBuf;
use crate::db::crud;
use crate::db::models::DataOfUnhashedBook;
use crate::db::models::{Book, BookDataType, DataOfHashedBook};
use crate::services::dir_scan_service::book_separator::BookSeparator;
use crate::types::{BookHash, BookPath, BookSize};
use crate::utils::calc_file_hash;
use gxhash::{HashMap, HashMapExt};
use measure_time_macro::measure_time;
use rayon::prelude::*;
use tracing::{event, info, Level};


pub(crate) async fn run(book_separator: BookSeparator) {
  let start = std::time::Instant::now();

  let (unique_books, list_of_same_size_books) = book_separator.separate();

  info!("Len of unique size books: {:?}", unique_books.books.len());
  crud::insert_batch::<Book>(unique_books.books);
  crud::insert_batch::<DataOfUnhashedBook>(unique_books.data);
  info!("Time to add unique size books: {:?}", start.elapsed());

  let (hashed_books, data_for_hashed_books) =
    tokio::spawn(get_hashed_books_and_their_data(list_of_same_size_books)).await.unwrap();

  info!("Len of repeating size books: {:?}", hashed_books.len());
  crud::insert_batch::<Book>(hashed_books);
  crud::insert_batch::<DataOfHashedBook>(data_for_hashed_books);
  info!("Time to add new books is: {:?}", start.elapsed());
}

#[measure_time]
async fn get_hashed_books_and_their_data(list_of_books_sizes: Vec<(BookSize, Vec<PathBuf>)>) -> (Vec<Book>, Vec<DataOfHashedBook>) {
  let mut book_hash_map: HashMap<BookHash, (BookSize, Vec<BookPath>)> = HashMap::new();
  let mut list_hashed_books: Vec<Book> = vec![];
  let mut data_for_hashed_books: Vec<DataOfHashedBook> = vec![];
  let start = std::time::Instant::now();
  
  for (book_size, paths_to_books) in list_of_books_sizes {
    
    let hashed_books: Vec<(BookHash, Book)> = paths_to_books.into_par_iter().map(|book_path| {
      let book_hash = calc_file_hash(book_path.to_str().unwrap());
      let new_book = Book::from_pathbuf(&book_path, BookDataType::RepeatingSize(book_hash.clone()));
      (book_hash, new_book)
    }).collect();
    
    for (book_hash, book) in hashed_books {
      match book_hash_map.get_mut(&book_hash) {
        None => {
          book_hash_map.insert(book_hash, (book_size.clone(), vec![book.path_to_book.clone()]));
        }
        Some((_book_size, books)) => {
          books.push(book.path_to_book.clone())
        }
      }
      list_hashed_books.push(book);
    }
  }

  for (book_hash, (book_size, books)) in book_hash_map {
    data_for_hashed_books.push(DataOfHashedBook::new(book_hash, book_size.clone(), books));
  };

  info!("Time to calc book hashes: {:?}", start.elapsed());
  (list_hashed_books, data_for_hashed_books)
}

