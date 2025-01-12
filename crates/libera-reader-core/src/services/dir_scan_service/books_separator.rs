use crate::models::Book;
use crate::types::BookPath;
use crate::utils::get_books_from_disk;
use gxhash::{HashMap, HashSet};
use std::path::PathBuf;
use tracing::debug;


pub(crate) struct BookSeparator {
  pub(crate) new_books: HashSet<PathBuf>,
  pub(crate) general_books: HashSet<Book>,
  pub(crate) outdated_books: Vec<Book>,
  pub(crate) num_of_books_on_disk: usize,
  pub(crate) num_of_books_in_db: usize,
}

impl BookSeparator {
  pub(crate) fn new(path_to_scan: &BookPath) -> Self {
    let mut books_on_disk: HashMap<BookPath, PathBuf> = get_books_from_disk(path_to_scan)
      .into_iter().map(|i| (i.to_str().unwrap().to_string(), i)).collect();
    let mut books_in_db: HashMap<BookPath, Book> = Book::get_all_from_db().into_iter()
      .map(|i| (i.path_to_book.clone(), i)).collect();

    let books_paths_on_disk: HashSet<BookPath> = books_on_disk.keys().cloned().collect();
    let books_paths_in_db: HashSet<BookPath> = books_in_db.keys().cloned().collect();
    let num_of_books_on_disk = books_on_disk.len();
    let num_of_books_in_db = books_in_db.len();

    let new_books: HashSet<PathBuf> = books_paths_on_disk.difference(&books_paths_in_db).map(|i| {
      books_on_disk.remove(i).unwrap()
    }).collect();
    let general_books: HashSet<Book> = books_paths_on_disk.intersection(&books_paths_in_db).map(|i| {
      books_in_db.remove(i).unwrap()
    }).collect();
    let outdated_books: Vec<Book> = books_paths_in_db.difference(&books_paths_on_disk).map(|book_path|
      books_in_db.remove(book_path).unwrap()
    ).collect();

    debug!("Number of new books: {:?}", new_books.len());
    debug!("Number of general_books: {:?}", general_books.len());
    debug!("Number of outdated books: {:?}", outdated_books.len());
    Self {
      new_books,
      general_books,
      outdated_books,
      num_of_books_on_disk,
      num_of_books_in_db,
    }
  }
}
