use crate::db::crud;
use crate::db::models::{Book, DataOfHashedBook, DataOfUnhashedBook};
use crate::services::dir_scan_service::books_separator::{HashedBooks, UniqueBooks};
use tracing::debug;


pub(crate) async fn run(unique_books: UniqueBooks, hashed_books: HashedBooks) {
  let start_time = std::time::Instant::now();

  debug!("Len of unique size books: {:?}", unique_books.books.len());
  crud::insert_batch::<Book>(unique_books.books);
  crud::insert_batch::<DataOfUnhashedBook>(unique_books.data);
  debug!("Time to add unique size books: {:?}", start_time.elapsed());


  debug!("Len of repeating size books: {:?}", hashed_books.books.len());
  crud::insert_batch::<Book>(hashed_books.books);
  crud::insert_batch::<DataOfHashedBook>(hashed_books.data);
  debug!("Time to add new books is: {:?}", start_time.elapsed());
}
