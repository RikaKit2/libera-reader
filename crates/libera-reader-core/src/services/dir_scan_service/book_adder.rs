use gxhash::{HashMap, HashMapExt};
use crate::db::crud;
use crate::db::models::{Book, BookDataType, DataOfHashedBook};
use crate::db::models::DataOfUnhashedBook;
use tracing::{info, event, Level};
use measure_time_macro::measure_time;
use crate::services::dir_scan_service::BookSizeMap;
use crate::utils::{calc_file_hash, BookHash, BookPath, BookSize};


pub(crate) async fn run(book_size_map: BookSizeMap) {
  let start = std::time::Instant::now();

  let (unique_size_books, unique_size_book_data, list_of_books_sizes)
    = book_size_map.get_books_repeating_and_uniquely_size();

  info!("Len of unique size books: {:?}", unique_size_books.len());
  crud::insert_batch::<Book>(unique_size_books);
  crud::insert_batch::<DataOfUnhashedBook>(unique_size_book_data);
  info!("Time to add unique size books: {:?}", start.elapsed());

  let (hashed_books, data_for_hashed_books) =
    tokio::spawn(get_hashed_books_and_their_data(list_of_books_sizes)).await.unwrap();

  info!("Len of repeating size books: {:?}", hashed_books.len());
  crud::insert_batch::<Book>(hashed_books);
  crud::insert_batch::<DataOfHashedBook>(data_for_hashed_books);
  info!("Time to add new books is: {:?}", start.elapsed());
}

#[measure_time]
pub async fn get_hashed_books_and_their_data(list_of_books_sizes: Vec<(BookSize, Vec<Book>)>) -> (Vec<Book>, Vec<DataOfHashedBook>) {
  let mut book_hash_map: HashMap<BookHash, (BookSize, Vec<BookPath>)> = HashMap::new();
  let mut list_hashed_books: Vec<Book> = vec![];
  let mut data_for_hashed_books: Vec<DataOfHashedBook> = vec![];
  let start = std::time::Instant::now();

  for (book_size, books) in list_of_books_sizes {
    for mut book in books {
      let book_hash = calc_file_hash(&book.path_to_book);
      book.book_data_pk = Some(BookDataType::RepeatingSize(Some(book_hash.clone())));
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

