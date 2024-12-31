use std::path::PathBuf;
use crate::models::Book;
use crate::services::dir_scan_service::book_separator::BookSeparator;
use crate::types::BookPath;
use gxhash::{HashMap, HashMapExt};
use measure_time_macro::measure_time;
use tracing::{debug, info};
use walkdir::WalkDir;

mod book_separator;
mod book_deleter;
mod book_adder;


enum BooksLocation {
  Disk,
  DB,
  Both,
  None,
}


pub(crate) async fn run(path_to_scan: BookPath) {
  let start = std::time::Instant::now();

  let books_on_disk: HashMap<BookPath, PathBuf> = get_books_from_disk(&path_to_scan);
  let books_in_db: HashMap<BookPath, Book> = Book::get_all_from_db().into_iter().map(|i| (i.path_to_book.clone(), i)).collect();

  let books_on_disk_len = books_on_disk.len();
  let books_in_db_len = books_in_db.len();

  let mut book_separator: BookSeparator = BookSeparator::new();

  let outdated_books = book_separator.fill_and_get_outdated_books(books_on_disk, books_in_db);

  match get_books_location(books_in_db_len, books_on_disk_len) {
    BooksLocation::Disk => {
      book_adder::run(book_separator).await;
    }
    BooksLocation::DB => {
      book_adder::run(book_separator).await;
    }
    BooksLocation::Both => {
      book_adder::run(book_separator).await;
      book_deleter::del_outdated_books(outdated_books);
    }
    BooksLocation::None => {}
  };
  info!("Dir scan service execution time is: {:?}", start.elapsed());
}

#[measure_time]
fn get_books_from_disk(path_to_scan: &String) -> HashMap<BookPath, PathBuf> {
  let mut books_from_disk: HashMap<BookPath, PathBuf> = HashMap::new();
  for entry in WalkDir::new(path_to_scan) {
    let entry = entry.unwrap();
    if entry.file_type().is_file() {
      let path = entry.path();
      match path.extension() {
        Some(res) => {
          let file_ext = res.to_str().unwrap();
          if ["pdf"].contains(&file_ext) {
            books_from_disk.insert(path.to_str().unwrap().to_string(), path.to_path_buf());
          }
        }
        None => {}
      };
    }
  };
  books_from_disk
}

#[measure_time]
fn get_books_location(db_book_count: usize, disk_book_count: usize) -> BooksLocation {
  if db_book_count > 0 && disk_book_count == 0 {
    BooksLocation::DB
  } else if db_book_count == 0 && disk_book_count > 0 {
    BooksLocation::Disk
  } else if db_book_count > 0 && disk_book_count > 0 {
    BooksLocation::Both
  } else {
    BooksLocation::None
  }
}

