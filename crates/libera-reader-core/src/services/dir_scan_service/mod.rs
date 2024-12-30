use crate::db::crud;
use crate::db::models::{Book, BookDataType, DataOfHashedBook, DataOfUnhashedBook};
use crate::types::{BookPath, BookSize, BooksCount};
use crate::utils::calc_file_size_in_mb;
use gxhash::{HashMap, HashMapExt, HashSet};
use itertools::Itertools;
use measure_time_macro::measure_time;
use tracing::{debug, info};
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

  let books_on_disk: HashMap<BookPath, Book> = get_books_from_disk(&path_to_scan);
  let books_in_db: HashMap<BookPath, Book> = Book::get_all_from_db().into_iter().map(|i| (i.path_to_book.clone(), i)).collect();

  let books_on_disk_len = books_on_disk.len();
  let books_in_db_len = books_in_db.len();

  let mut book_size_map: BooksSizeMap = BooksSizeMap::new();

  let _old_books = get_outdated_books(books_on_disk, books_in_db, &mut book_size_map);

  match get_books_location(books_in_db_len, books_on_disk_len) {
    BooksLocation::Disk => {
      book_adder::run(book_size_map).await;
    }
    BooksLocation::DB => {
      book_adder::run(book_size_map).await;
    }
    BooksLocation::Both => {
      book_adder::run(book_size_map).await;
      // if outdated_books.len() > 0 {
      // TODO: make deleting outdated_books
      // }
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
fn get_outdated_books(books_on_disk: HashMap<BookPath, Book>, mut books_in_db: HashMap<BookPath, Book>,
                      book_size_map: &mut BooksSizeMap) -> Vec<Book> {
  let books_paths_on_disk: HashSet<BookPath> = books_on_disk.keys().map(|i| i.clone()).collect();
  let books_paths_in_db: HashSet<BookPath> = books_in_db.keys().map(|i| i.clone()).collect();

  let existing_paths_to_books = books_paths_on_disk.intersection(&books_paths_in_db).collect_vec();
  let new_books_paths = books_paths_on_disk.difference(&books_paths_in_db).collect_vec();
  let outdated_books: Vec<Book> = books_paths_in_db.difference(&books_paths_on_disk)
    .map(|i| books_in_db.remove(i).unwrap()).collect_vec();

  info!("Number of new books: {:?}", new_books_paths.len());
  info!("Number of existing books: {:?}", existing_paths_to_books.len());
  info!("Number of outdated books: {:?}", outdated_books.len());

  book_size_map.extend_overall_books(existing_paths_to_books, books_in_db);
  book_size_map.extend_new_books(new_books_paths, books_on_disk);
  outdated_books
}

pub(crate) struct BooksSizeMap {
  new_books: HashMap<BookSize, Vec<Book>>,
  existing_books: HashMap<BookSize, BooksCount>,
}
impl BooksSizeMap {
  pub fn new() -> Self { Self { new_books: Default::default(), existing_books: Default::default() } }
  pub(crate) fn extend_new_books(&mut self, new_books_paths: Vec<&BookPath>, mut books_on_disk: HashMap<BookPath, Book>) {
    for book_path in new_books_paths {
      let new_book = books_on_disk.remove(book_path).unwrap();
      let book_size = calc_file_size_in_mb(book_path).to_string();
      match self.new_books.get_mut(&book_size) {
        None => { self.new_books.insert(book_size, vec![new_book]); }
        Some(group_of_books) => { group_of_books.push(new_book); }
      }
    }
  }
  pub fn extend_overall_books(&mut self, overall_books: Vec<&BookPath>, books_in_db: HashMap<BookPath, Book>) {
    for book_path in overall_books {
      let existing_book = books_in_db.get(book_path).unwrap().clone();
      let book_size: BookSize = match existing_book.book_data_pk.unwrap() {
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
  pub fn get_books_repeating_and_uniquely_size(mut self) -> (Vec<Book>, Vec<DataOfUnhashedBook>, Vec<(BookSize, Vec<Book>)>) {
    let mut unique_size_books: Vec<Book> = vec![];
    let mut data_unhashed_books: Vec<DataOfUnhashedBook> = vec![];
    let mut list_of_books_sizes: Vec<(BookSize, Vec<Book>)> = vec![];
    for (book_size, mut new_books) in self.new_books {
      let num_of_existing_books = self.existing_books.remove(&book_size).unwrap_or_else(|| { 0 });
      let total_num_of_books_of_this_size = num_of_existing_books + new_books.len();

      if total_num_of_books_of_this_size == 1 {
        let primary_keys: Vec<BookPath> = new_books.iter().map(|i| i.path_to_book.clone()).collect_vec();
        for book in new_books.iter_mut() {
          book.book_data_pk = Some(BookDataType::UniqueSize(book_size.clone()));
        }
        unique_size_books.extend(new_books);
        data_unhashed_books.push(DataOfUnhashedBook::new(book_size, primary_keys))
      } else if total_num_of_books_of_this_size > 1 {
        list_of_books_sizes.push((book_size, new_books));
      }
    }
    (unique_size_books, data_unhashed_books, list_of_books_sizes)
  }
}

