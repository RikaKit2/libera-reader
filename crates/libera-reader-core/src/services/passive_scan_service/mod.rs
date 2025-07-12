use crate::db::models::Book;
use crate::services::Services;
use crate::types::HashSet;
use anyhow::Result;
use books_separator::BookSeparator;
use tracing::{debug, info};

mod book_adder;
mod book_deleter;
mod books_separator;

enum BooksLocation {
  Disk,
  DB,
  DiskAndDB,
  None,
}
impl Services {
  pub fn run_passive_scan(&mut self, path_to_scan: &String) -> Result<()> {
    let book_separator = BookSeparator::new(path_to_scan, &self.db, &self.target_ext);
    self.fill_storage_of_non_cached_books(book_separator.general_books)?;
    let start_time = std::time::Instant::now();
    match self.get_books_location(book_separator.num_of_books_in_db, book_separator.num_of_books_on_disk) {
      BooksLocation::Disk => {
        book_adder::run(book_separator.new_books, &self.db, self.not_cached_books.clone())?;
      }
      BooksLocation::DB => {
        book_deleter::del_outdated_books(book_separator.outdated_books, &self.db, &self.app_dirs)?;
      }
      BooksLocation::DiskAndDB => {
        book_deleter::del_outdated_books(book_separator.outdated_books, &self.db, &self.app_dirs)?;
        book_adder::run(book_separator.new_books, &self.db, self.not_cached_books.clone())?;
      }
      BooksLocation::None => {}
    };
    info!("Dir scan service execution time is: {:?}", start_time.elapsed());
    Ok(())
  }
  fn get_books_location(&self, db_book_count: usize, disk_book_count: usize) -> BooksLocation {
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
  fn fill_storage_of_non_cached_books(&mut self, general_books: HashSet<Book>) -> Result<()> {
    for book in general_books {
      let book_data = book.get_book_data(&self.db)?;
      if !book_data.cached && book_data.mutool_err.is_none() {
        self.not_cached_books.push(Box::new(book))?;
      }
    }
    debug!("Number of not_cached_books: {:?}", self.not_cached_books.len());
    Ok(())
  }
}