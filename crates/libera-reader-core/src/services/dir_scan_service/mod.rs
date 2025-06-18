use crate::db::models::Book;
use crate::types::{APP_DIRS, HashSet, NotCachedBooks, TARGET_EXT, DB};
use books_separator::BookSeparator;
use tracing::{debug, info};

mod book_deleter;
mod book_adder;
mod books_separator;

enum BooksLocation {
  Disk,
  DB,
  DiskAndDB,
  None,
}

pub(crate) fn run(path_to_scan: &String, db: &DB, target_ext: TARGET_EXT, app_dirs: APP_DIRS, not_cached_books: &NotCachedBooks) {
  let book_separator = BookSeparator::new(&path_to_scan, db, &target_ext);
  fill_storage_of_non_cached_books(book_separator.general_books, db, not_cached_books);
  let start_time = std::time::Instant::now();
  match get_books_location(book_separator.num_of_books_in_db, book_separator.num_of_books_on_disk) {
    BooksLocation::Disk => {
      book_adder::run(book_separator.new_books, db);
    }
    BooksLocation::DB => {
      book_deleter::del_outdated_books(book_separator.outdated_books, db, &app_dirs);
    }
    BooksLocation::DiskAndDB => {
      book_deleter::del_outdated_books(book_separator.outdated_books, db, &app_dirs);
      book_adder::run(book_separator.new_books, db);
    }
    BooksLocation::None => {}
  };
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
fn fill_storage_of_non_cached_books(general_books: HashSet<Book>, db: &DB, not_cached_books: &NotCachedBooks) {
  for book in general_books {
    let book_data = book.get_book_data(db);
    if !book_data.cached && book_data.mutool_err.is_none() {
      not_cached_books.push(Box::new(book)).unwrap();
    }
  }
  debug!("Number of not_cached_books: {:?}", not_cached_books.len());
}
