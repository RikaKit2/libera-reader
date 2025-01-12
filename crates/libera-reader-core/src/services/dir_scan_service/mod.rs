use crate::services::data_extraction_service;
use crate::types::BookPath;
use books_separator::BookSeparator;
use tracing::info;


mod book_deleter;
mod book_adder;
mod books_separator;

enum BooksLocation {
  Disk,
  DB,
  DiskAndDB,
  None,
}

pub(crate) async fn run(path_to_scan: BookPath) {
  let book_separator = BookSeparator::new(&path_to_scan);
  let start_time = std::time::Instant::now();
  match get_books_location(book_separator.num_of_books_in_db, book_separator.num_of_books_on_disk) {
    BooksLocation::Disk => {
      book_adder::run(book_separator.new_books).await;
    }
    BooksLocation::DB => {
      book_deleter::del_outdated_books(book_separator.outdated_books);
    }
    BooksLocation::DiskAndDB => {
      book_deleter::del_outdated_books(book_separator.outdated_books);
      book_adder::run(book_separator.new_books).await;
    }
    BooksLocation::None => {}
  };
  data_extraction_service::fill_storage_of_non_cached_books(book_separator.general_books);
  info!("Dir scan service execution time is: {:?}", start_time.elapsed());
}

fn get_books_location(db_book_count: usize, disk_book_count: usize) -> BooksLocation {
  if db_book_count > 0 && disk_book_count == 0 {
    BooksLocation::DB
  } else if db_book_count == 0 && disk_book_count > 0 {
    BooksLocation::Disk
  } else if db_book_count > 0 && disk_book_count > 0 {
    BooksLocation::DiskAndDB
  } else {
    BooksLocation::None
  }
}

