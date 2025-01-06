use crate::db::crud;
use crate::models::{Book, BookDataType, DataOfHashedBook, DataOfUnhashedBook};
use crate::types::{BookHash, BookPath, BookSize};
use crate::utils::{calc_file_hash, calc_file_size_in_mb};
use gxhash::{HashMap, HashMapExt, HashSet};
use itertools::Itertools;
use rayon::prelude::*;
use std::path::PathBuf;
use tracing::debug;


pub(crate) struct UniqueBooks {
  pub(crate) books: Vec<Book>,
  pub(crate) data: Vec<DataOfUnhashedBook>,
}
impl UniqueBooks {
  pub fn new() -> Self { Self { books: vec![], data: vec![] } }
}

pub(crate) struct HashedBooks {
  pub(crate) books: Vec<Book>,
  pub(crate) data: Vec<DataOfHashedBook>,
}
impl HashedBooks {
  pub fn new() -> Self { Self { books: vec![], data: vec![] } }
  pub(crate) async fn fill_and_run(&mut self, books_for_hashing: BooksForHashing) {
    if books_for_hashing.len() > 0 {
      let (hashed_books, data_of_hashed_books) = tokio::spawn(async {
        let mut out_books: Vec<Book> = vec![];
        let mut data_of_hashed_books: Vec<DataOfHashedBook> = vec![];

        let mut books_with_same_hash: HashMap<BookHash, (BookSize, Vec<BookPath>)> = HashMap::new();

        let start = std::time::Instant::now();
        for (book_size, paths_to_books) in books_for_hashing {
          let hashed_books: Vec<(BookHash, Book)> = paths_to_books.into_par_iter().map(|book_path| {
            let book_hash = calc_file_hash(book_path.to_str().unwrap());
            let new_book = Book::from_pathbuf(&book_path, BookDataType::RepeatingSize(book_hash.clone()));
            (book_hash, new_book)
          }).collect();

          // Grouping books by hash
          for (book_hash, book) in hashed_books {
            match books_with_same_hash.get_mut(&book_hash) {
              None => {
                books_with_same_hash.insert(book_hash, (book_size.clone(), vec![book.path_to_book.clone()]));
              }
              Some((_book_size, books)) => {
                books.push(book.path_to_book.clone())
              }
            }
            out_books.push(book);
          }
        }

        for (book_hash, (book_size, books)) in books_with_same_hash {
          data_of_hashed_books.push(DataOfHashedBook::new(book_hash, book_size.clone(), books));
        };

        debug!("Time to calc book hashes: {:?}", start.elapsed());
        (out_books, data_of_hashed_books)
      }).await.unwrap();
      self.books = hashed_books;
      self.data = data_of_hashed_books;
    }
  }
}

type DBBookCount = usize;
type BooksGroupedBySize = HashMap<BookSize, (Vec<PathBuf>, DBBookCount)>;
type BooksForHashing = Vec<(BookSize, Vec<PathBuf>)>;

pub async fn run(books_on_disk: HashMap<BookPath, PathBuf>,
                 mut books_in_db: HashMap<BookPath, Book>) -> (Vec<Book>, UniqueBooks, HashedBooks) {
  let books_paths_on_disk: HashSet<BookPath> = books_on_disk.keys().cloned().collect();
  let books_paths_in_db: HashSet<BookPath> = books_in_db.keys().cloned().collect();

  let new_books_paths = books_paths_on_disk.difference(&books_paths_in_db).collect_vec();
  let outdated_books: Vec<Book> = books_paths_in_db.difference(&books_paths_on_disk)
    .map(|i| books_in_db.remove(i).unwrap()).collect_vec();

  debug!("Number of new books: {:?}", new_books_paths.len());
  debug!("Number of outdated books: {:?}", outdated_books.len());

  let (unique_books, books_for_hashing) = get_hashed_and_unique_books(books_on_disk, new_books_paths);

  let mut hashed_books = HashedBooks::new();
  hashed_books.fill_and_run(books_for_hashing).await;
  (outdated_books, unique_books, hashed_books)
}

fn get_books_grouped_by_size(mut books_on_disk: HashMap<BookPath, PathBuf>, new_books_paths: Vec<&BookPath>) -> BooksGroupedBySize {
  let mut books_grouped_by_size: BooksGroupedBySize = HashMap::new();

  for book_path in new_books_paths {
    let new_book_path = books_on_disk.remove(book_path).unwrap();
    let book_size = calc_file_size_in_mb(book_path).to_string();

    let db_book_count = crud::get_primary::<DataOfUnhashedBook>(book_size.clone())
      .map_or(0, |res| res.book_data.books_pk.len());

    match books_grouped_by_size.get_mut(&book_size) {
      None => {
        books_grouped_by_size.insert(book_size, (vec![new_book_path], db_book_count));
      }
      Some(new_books_vec) => { new_books_vec.0.push(new_book_path); }
    }
  }
  books_grouped_by_size
}

fn get_hashed_and_unique_books(books_on_disk: HashMap<BookPath, PathBuf>,
                               new_books_paths: Vec<&BookPath>) -> (UniqueBooks, BooksForHashing) {
  let mut books_for_hashing: BooksForHashing = vec![];
  let mut unique_books = UniqueBooks::new();
  if new_books_paths.len() > 0 {
    for (book_size, (books_paths, db_book_count)) in get_books_grouped_by_size(books_on_disk, new_books_paths) {
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
  }
  (unique_books, books_for_hashing)
}

