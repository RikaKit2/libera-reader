mod books_location;

use std::path::PathBuf;

use anyhow::Result;
use jwalk::WalkDir;

use utils::debug;

use crate::{
  db::{
    DB,
    models::books::{
      Books,
      book::{BookDir, BookPath},
    },
  },
  error_handler::ErrorHandler,
  not_cached_books::NotCachedBooks,
  services::notify_service::fs_handlers,
  settings::SETTINGS,
  types::{HashMap, HashSet},
};
use books_location::BooksLocation;

pub(crate) type BooksFromDB = HashMap<BookDir, Books>;
pub(crate) struct BooksFromDisk {
  inner: HashMap<BookDir, HashSet<BookPath>>,
  books_count: usize,
}

impl BooksFromDisk {
  pub(crate) fn new() -> Self {
    Self { inner: Default::default(), books_count: 0 }
  }

  pub(crate) fn insert_book_path(&mut self, book_path: BookPath) {
    let set = self.inner.entry(book_path.parent_dir.clone()).or_default();
    if set.insert(book_path) {
      self.books_count += 1;
    }
  }
  pub(crate) fn len(&self) -> usize {
    self.books_count
  }
  pub(crate) fn get(&self, key: &BookDir) -> Option<&HashSet<BookPath>> {
    self.inner.get(key)
  }
}

impl IntoIterator for BooksFromDisk {
  type Item = (BookDir, HashSet<BookPath>);
  type IntoIter = indexmap::map::IntoIter<BookDir, HashSet<BookPath>>;

  fn into_iter(self) -> Self::IntoIter {
    self.inner.into_iter()
  }
}

pub(crate) type DBBooksCount = usize;

pub struct ScanService {
  not_cached_books: NotCachedBooks,
  settings: SETTINGS,
  db: DB,
  _error_handler: ErrorHandler,
}

impl ScanService {
  pub(crate) fn new(not_cached_books: NotCachedBooks, settings: SETTINGS, db: DB, error_handler: ErrorHandler) -> Self {
    Self { not_cached_books, settings, db, _error_handler: error_handler }
  }
  pub async fn run(&mut self) -> Result<()> {
    match self.settings.get_path_to_scan_if_exists() {
      None => {
        debug!("Path to scan is not set. Please set it in the settings.");
      }
      Some(path_to_scan) => {
        let not_cached_books = self.not_cached_books.clone();
        let settings = self.settings.clone();
        let db = self.db.clone();

        let start_time = std::time::Instant::now();
        let books_from_disk = Self::get_books_from_disk(&path_to_scan, &settings);

        match BooksLocation::classify(&db, books_from_disk.len()).unwrap() {
          BooksLocation::Disk => {
            for (_book_dir, set) in books_from_disk.into_iter() {
              for book_path in set {
                fs_handlers::insert_book(book_path, &db, &settings, &not_cached_books).await.unwrap();
              }
            }
          }
          BooksLocation::DB(books_from_db) => {
            for (_book_dir, books) in books_from_db {
              books.remove_self(&db).unwrap();
            }
          }
          BooksLocation::DiskAndDB(books_from_db) => {
            Self::remove_outdated_books_from_db(&db, &books_from_disk, books_from_db).await.unwrap();
            Self::insert_new_books_to_db(&db, books_from_disk).await.unwrap();
          }
          BooksLocation::None => {}
        };
        debug!("Total time of executing scan_service: {:?}", start_time.elapsed());
      }
    };
    Ok(())
  }
  fn get_books_from_disk(path_to_scan: &PathBuf, settings: &SETTINGS) -> BooksFromDisk {
    let start_time = std::time::Instant::now();
    let mut books_from_disk: BooksFromDisk = BooksFromDisk::new();
    for entry in WalkDir::new(path_to_scan) {
      if let Ok(entry) = entry
        && entry.file_type().is_file()
      {
        let path = entry.path();
        if path.extension().is_some()
          && let Some(file_path) = BookPath::new(&path)
          && settings.contains_ext(&file_path.ext)
        {
          books_from_disk.insert_book_path(file_path);
        }
      }
    }
    debug!("The total time of receiving books from the disk: {:?}", start_time.elapsed());
    books_from_disk
  }
  async fn remove_outdated_books_from_db(db: &DB, books_from_disk: &BooksFromDisk, books_from_db: BooksFromDB) -> anyhow::Result<()> {
    let mut num_of_outdated_books: usize = 0;
    for (book_dir, books) in books_from_db {
      match books_from_disk.get(&book_dir) {
        Some(disk_books) => {
          for (_book_name, book) in books.storage.iter() {
            match disk_books.contains(&book.book_path) {
              true => {}
              false => {
                num_of_outdated_books += 1;
                books.remove_book(book.book_path.clone(), db).await.unwrap();
              }
            };
          }
        }
        None => {
          num_of_outdated_books += books.storage.len();
          books.remove_self(db).unwrap();
        }
      };
    }
    debug!("Number of outdated books: {:?}", &num_of_outdated_books);
    Ok(())
  }
  async fn insert_new_books_to_db(db: &DB, books_from_disk: BooksFromDisk) -> anyhow::Result<()> {
    let mut books_from_db: (BooksFromDB, DBBooksCount) = Books::all(db);
    let mut num_of_new_books: usize = 0;
    for (book_dir, disk_books) in books_from_disk.into_iter() {
      match books_from_db.0.get_mut(&book_dir) {
        Some(db_books) => {
          let mut new_books = vec![];
          for book_path in disk_books {
            match db_books.storage.contains_key(&book_path.name) {
              true => {}
              false => {
                new_books.push(book_path);
              }
            };
          }
          num_of_new_books += new_books.len();
          db_books.insert_many(new_books.into_iter(), db).await?;
        }
        None => {
          num_of_new_books += disk_books.len();
          Books::insert_many_and_create_new_self(book_dir, disk_books.into_iter(), db).await?;
        }
      };
    }
    debug!("Number of new books: {:?}", &num_of_new_books);
    Ok(())
  }
}
