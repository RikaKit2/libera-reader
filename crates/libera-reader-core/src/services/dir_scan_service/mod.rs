use crate::db::models::Book;
use crate::utils::BookPath;
use gxhash::{HashMap, HashMapExt, HashSet, HashSetExt};
use measure_time_macro::measure_time;
use std::ops::Sub;
use tracing::info;
use walkdir::WalkDir;

mod book_adder;


enum BooksLocation {
  Disk,
  DB,
  Both,
  None,
}

pub(crate) async fn run(path_to_scan: String) {
  let start = std::time::Instant::now();
  let db_books_set: HashMap<BookPath, Book> = Book::get_all_from_db().into_iter().map(
    |i| (i.path_to_book.clone(), i)).collect();
  let disk_books_set: HashMap<BookPath, Book> = get_books_from_disk(&path_to_scan);

  info!("Number of books in the db: {:?}", db_books_set.len());
  let db_book_count = db_books_set.len();
  let disk_book_count = disk_books_set.len();

  let (new_books, outdated_books) = get_new_and_outdated_books(disk_books_set, db_books_set);

  info!("New books count: {:?}", new_books.len());
  info!("Outdated books count: {:?}", outdated_books.len());

  match get_books_location(db_book_count, disk_book_count) {
    BooksLocation::Disk => {
      book_adder::run(new_books).await;
    }
    BooksLocation::DB => {
      book_adder::run(new_books).await;
    }
    BooksLocation::Both => {
      book_adder::run(new_books).await;
      if outdated_books.len() > 0 {
        todo!("make deleting outdated_books")
      }
    }
    BooksLocation::None => {}
  };
  info!("Dir scan service execution time is: {:?}", start.elapsed());
}

#[measure_time]
fn get_books_from_disk(path_to_scan: &String) -> HashMap<BookPath, Book> {
  let mut books_from_disk: HashMap<BookPath, Book> = HashMap::new();
  for entry in WalkDir::new(path_to_scan) {
    let entry = entry.unwrap();
    if entry.file_type().is_file() {
      let path = entry.path();
      match path.extension() {
        Some(res) => {
          let file_ext = res.to_str().unwrap();
          if ["pdf"].contains(&file_ext) {
            let book = Book::from_pathbuf(&path.to_path_buf());
            books_from_disk.insert(book.path_to_book.clone(), book);
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

#[measure_time]
fn get_new_and_outdated_books(mut disk_books: HashMap<BookPath, Book>,
                              mut db_books_set: HashMap<BookPath, Book>) -> (Vec<Book>, Vec<Book>) {
  let mut disk_books_paths_set: HashSet<BookPath> = HashSet::new();
  let mut db_books_paths_set: HashSet<BookPath> = HashSet::new();

  let books_paths_from_disk: Vec<BookPath> = disk_books.keys().into_iter()
    .map(|i| i.clone()).collect();
  disk_books_paths_set.extend(books_paths_from_disk);
  let books_paths_from_db: Vec<BookPath> = db_books_set.keys().into_iter()
    .map(|i| i.clone()).collect();
  db_books_paths_set.extend(books_paths_from_db);

  let new_books_set = disk_books_paths_set.sub(&db_books_paths_set);
  let outdated_book_set = db_books_paths_set.sub(&disk_books_paths_set);

  let mut new_books: Vec<Book> = vec![];
  let mut outdated_books: Vec<Book> = vec![];
  for i in new_books_set {
    new_books.push(disk_books.remove(&i).unwrap());
  }
  for i in outdated_book_set {
    outdated_books.push(db_books_set.remove(&i).unwrap());
  }
  (new_books, outdated_books)
}

