use crate::db::crud;
use crate::db::models::{Book, BookDataWrapperPK, DataOfUnhashedBook};
use crate::types::{BookPath, BookSize, HashMap, HashSet, NotCachedBooks, DB};
use crate::utils::RayonTask;
use anyhow::Result;
use itertools::Itertools;
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use std::path::PathBuf;
use tracing::debug;
use utils::FileSizeMeasure;

type DBBookCount = usize;
type BooksGroupedBySize = HashMap<BookSize, (Vec<PathBuf>, DBBookCount)>;
type BooksForHashing = Vec<(BookSize, Vec<PathBuf>)>;

pub(crate) fn run(new_books: HashSet<PathBuf>, db: &DB, not_cached_books: NotCachedBooks) -> Result<Vec<Result<()>>> {
  let start_time = std::time::Instant::now();
  let num_of_new_books = new_books.len();
  let books_grouped_by_size = get_books_grouped_by_size(new_books, db)?;
  let (unique_books, books_for_hashing) = get_hashed_and_unique_books(books_grouped_by_size);
  let num_of_unique_books = unique_books.books.len();
  let mut poss_errors: Vec<Result<()>> = vec![];

  debug!("Number of books for hashing: {:?}", num_of_new_books - num_of_unique_books);
  debug!("Number of books of a unique size: {:?}", num_of_unique_books);
  crud::insert_batch::<Book>(unique_books.books, db)?;
  crud::insert_batch::<DataOfUnhashedBook>(unique_books.data, db)?;
  debug!("Time to add unique size books: {:?}", start_time.elapsed());

  let num_of_threads = RayonTask::CalcHash.get_num_of_threads();
  debug!("Number of threads for hash calculation: {:?}", &num_of_threads);
  ThreadPoolBuilder::new().num_threads(num_of_threads).build()?.install(|| {
    for (book_size, books) in books_for_hashing {
      let errors: Vec<Result<()>> = books.par_iter().map(|book_pathbuf|
        crud::book::add_book(book_pathbuf, book_size.clone(), db, &not_cached_books)).collect();
      poss_errors.extend(errors);
    }
  });
  Ok(poss_errors)
}
fn get_books_grouped_by_size(new_books: HashSet<PathBuf>, db: &DB) -> Result<BooksGroupedBySize> {
  let mut books_grouped_by_size: BooksGroupedBySize = HashMap::default();

  for new_book_path in new_books {
    let book_size = FileSizeMeasure::MB.get_file_size(&new_book_path, 2)?.to_string();
    let (db_book_count, _) = crud::book::get_num_of_books_of_this_size(book_size.clone(), db)?;

    match books_grouped_by_size.get_mut(&book_size) {
      None => {
        books_grouped_by_size.insert(book_size, (vec![new_book_path], db_book_count));
      }
      Some(new_books_vec) => {
        new_books_vec.0.push(new_book_path);
      }
    }
  }
  Ok(books_grouped_by_size)
}
fn get_hashed_and_unique_books(books_grouped_by_size: BooksGroupedBySize) -> (UniqueBooks, BooksForHashing) {
  let mut books_for_hashing: BooksForHashing = vec![];
  let mut unique_books = UniqueBooks::new();
  for (book_size, (books_paths, db_book_count)) in books_grouped_by_size {
    let num_books_of_this_size = db_book_count + books_paths.len();

    if num_books_of_this_size == 1 {
      let primary_keys: Vec<BookPath> = books_paths.iter().map(|i| i.to_str().unwrap().to_string()).collect_vec();

      let new_books = books_paths.iter().map(|book_path|
        Book::from_pathbuf(book_path, BookDataWrapperPK::UniqueSize(book_size.clone()))).collect_vec();

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
  pub fn new() -> Self {
    Self {
      books: vec![],
      data: vec![],
    }
  }
}
