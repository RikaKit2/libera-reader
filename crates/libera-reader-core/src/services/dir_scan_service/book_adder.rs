use crate::db::crud;
use crate::models::{Book, BookDataType, DataOfUnhashedBook};
use crate::types::{BookPath, BookSize};
use crate::utils::RayonTaskType::HashCalc;
use crate::utils::{calc_file_size_in_mb, get_num_of_threads};
use gxhash::{HashMap, HashMapExt, HashSet};
use itertools::Itertools;
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use std::path::PathBuf;
use tracing::debug;

type DBBookCount = usize;
type BooksGroupedBySize = HashMap<BookSize, (Vec<PathBuf>, DBBookCount)>;
type BooksForHashing = Vec<(BookSize, Vec<PathBuf>)>;


pub(crate) fn run(new_books: HashSet<PathBuf>) {
  let start_time = std::time::Instant::now();
  let num_of_new_books = new_books.len();
  let books_grouped_by_size = get_books_grouped_by_size(new_books);
  let (unique_books, books_for_hashing) = get_hashed_and_unique_books(books_grouped_by_size);
  let num_of_unique_books = unique_books.books.len();

  debug!("Number of books for hashing: {:?}", num_of_new_books - num_of_unique_books);
  debug!("Number of books of a unique size: {:?}", num_of_unique_books);
  crud::insert_batch::<Book>(unique_books.books);
  crud::insert_batch::<DataOfUnhashedBook>(unique_books.data);
  debug!("Time to add unique size books: {:?}", start_time.elapsed());

  let num_of_threads = get_num_of_threads(HashCalc);
  debug!("Number of threads for hash calculation: {:?}", &num_of_threads);
  ThreadPoolBuilder::new().num_threads(num_of_threads).build().unwrap().install(|| {
    for (book_size, books) in books_for_hashing {
      books.par_iter().for_each(|bookbuf| crud::book::add_book(bookbuf, book_size.clone()));
    }
  });
}

fn get_books_grouped_by_size(new_books: HashSet<PathBuf>) -> BooksGroupedBySize {
  let mut books_grouped_by_size: BooksGroupedBySize = HashMap::new();

  for new_book_path in new_books {
    let book_size = calc_file_size_in_mb(&new_book_path);
    let db_book_count = match crud::get_primary::<DataOfUnhashedBook>(book_size.clone()) {
      None => { 0 }
      Some(res) => { res.book_data.books_pk.len() }
    };

    match books_grouped_by_size.get_mut(&book_size) {
      None => {
        books_grouped_by_size.insert(book_size, (vec![new_book_path], db_book_count));
      }
      Some(new_books_vec) => { new_books_vec.0.push(new_book_path); }
    }
  }
  books_grouped_by_size
}
fn get_hashed_and_unique_books(books_grouped_by_size: BooksGroupedBySize) -> (UniqueBooks, BooksForHashing) {
  let mut books_for_hashing: BooksForHashing = vec![];
  let mut unique_books = UniqueBooks::new();
  for (book_size, (books_paths, db_book_count)) in books_grouped_by_size {
    let num_books_of_this_size = db_book_count + books_paths.len();

    if num_books_of_this_size == 1 {
      let primary_keys: Vec<BookPath> = books_paths.iter().map(|i| i.to_str().unwrap().to_string()).collect_vec();

      let new_books = books_paths.iter().map(|book_path| {
        Book::from_pathbuf(book_path, BookDataType::UniqueSize(book_size.clone()))
      }).collect_vec();

      unique_books.books.extend(new_books);
      unique_books.data.push(DataOfUnhashedBook::new(book_size, primary_keys));
    } else if num_books_of_this_size > 1 {
      books_for_hashing.push((book_size, books_paths));
    }
  }
  (unique_books, books_for_hashing)
}

struct UniqueBooks {
  pub(crate) books: Vec<Book>,
  pub(crate) data: Vec<DataOfUnhashedBook>,
}
impl UniqueBooks {
  pub fn new() -> Self { Self { books: vec![], data: vec![] } }
}
