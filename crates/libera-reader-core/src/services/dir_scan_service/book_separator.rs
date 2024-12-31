use crate::db::crud;
use crate::models::{Book, BookDataType, DataOfHashedBook, DataOfUnhashedBook};
use crate::types::{BookPath, BookSize, BooksCount};
use crate::utils::calc_file_size_in_mb;
use gxhash::{HashMap, HashSet};
use itertools::Itertools;
use std::path::PathBuf;
use tracing::info;


pub(crate) struct UniqueBooks {
  pub(crate) books: Vec<Book>,
  pub(crate) data: Vec<DataOfUnhashedBook>,
}
impl UniqueBooks {
  pub fn new() -> Self { Self { books: vec![], data: vec![] } }
}
pub(crate) struct BookSeparator {
  new_books_paths: HashMap<BookSize, Vec<PathBuf>>,
  existing_books: HashMap<BookSize, BooksCount>,
}
impl BookSeparator {
  pub fn new() -> Self { Self { new_books_paths: Default::default(), existing_books: Default::default() } }
  pub fn fill_and_get_outdated_books(&mut self, books_on_disk: HashMap<BookPath, PathBuf>, mut books_in_db: HashMap<BookPath, Book>) -> Vec<Book> {
    let books_paths_on_disk: HashSet<BookPath> = books_on_disk.keys().map(|i| i.clone()).collect();
    let books_paths_in_db: HashSet<BookPath> = books_in_db.keys().map(|i| i.clone()).collect();

    let existing_paths_to_books = books_paths_on_disk.intersection(&books_paths_in_db).collect_vec();
    let new_books_paths = books_paths_on_disk.difference(&books_paths_in_db).collect_vec();

    let outdated_books: Vec<Book> = books_paths_in_db.difference(&books_paths_on_disk)
      .map(|i| books_in_db.remove(i).unwrap()).collect_vec();

    info!("Number of new books: {:?}", new_books_paths.len());
    info!("Number of existing books: {:?}", existing_paths_to_books.len());
    info!("Number of outdated books: {:?}", outdated_books.len());

    self.add_existing_books(existing_paths_to_books, books_in_db);
    self.add_new_books(new_books_paths, books_on_disk);
    outdated_books
  }
  fn add_new_books(&mut self, new_books_paths: Vec<&BookPath>, mut books_on_disk: HashMap<BookPath, PathBuf>) {
    for book_path in new_books_paths {
      let new_book_path = books_on_disk.remove(book_path).unwrap();
      let book_size = calc_file_size_in_mb(book_path).to_string();
      match self.new_books_paths.get_mut(&book_size) {
        None => { self.new_books_paths.insert(book_size, vec![new_book_path]); }
        Some(group_of_books) => { group_of_books.push(new_book_path); }
      }
    }
  }
  fn add_existing_books(&mut self, existing_books: Vec<&BookPath>, books_in_db: HashMap<BookPath, Book>) {
    for book_path in existing_books {
      let existing_book = books_in_db.get(book_path).unwrap().clone();
      let book_size: BookSize = match existing_book.book_data_pk {
        BookDataType::UniqueSize(i) => { i }
        BookDataType::RepeatingSize(i) => {
          let book_size: BookSize = crud::get_primary::<DataOfHashedBook>(i).unwrap().book_size;
          book_size
        }
      };
      match self.existing_books.get_mut(&book_size) {
        None => { self.existing_books.insert(book_size, 1); }
        Some(books_count) => { *books_count += 1; }
      }
    };
  }
  pub(crate) fn separate(mut self) -> (UniqueBooks, Vec<(BookSize, Vec<PathBuf>)>) {
    let mut unique_books = UniqueBooks::new();
    let mut list_of_same_size_books: Vec<(BookSize, Vec<PathBuf>)> = vec![];

    for (book_size, new_books_paths) in self.new_books_paths {
      let num_of_existing_books = self.existing_books.remove(&book_size).unwrap_or_else(|| { 0 });
      let num_books_of_this_size = num_of_existing_books + new_books_paths.len();

      if num_books_of_this_size == 1 {
        let primary_keys: Vec<BookPath> = new_books_paths.iter().map(|i| i.to_str().unwrap().to_string()).collect_vec();
        
        let new_books = new_books_paths.iter().map(|book_path| {
          Book::from_pathbuf(book_path, BookDataType::UniqueSize(book_size.clone()))
        }).collect_vec();
        
        unique_books.books.extend(new_books);
        unique_books.data.push(DataOfUnhashedBook::new(book_size, primary_keys));
      } else if num_books_of_this_size > 1 {
        list_of_same_size_books.push((book_size, new_books_paths));
      }
    }
    (unique_books, list_of_same_size_books)
  }
}
