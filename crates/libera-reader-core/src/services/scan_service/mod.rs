mod books_location;

use anyhow::Result;
use jwalk::WalkDir;
use std::{
  ops::{Deref, DerefMut},
  thread,
};
use tokio::runtime::Builder;
use tracing::info;

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
  services::{WorkStatus, notify_service::fs_handlers},
  settings::SETTINGS,
  types::{HashMap, HashSet},
};
use books_location::BooksLocation;

pub(crate) type BooksFromDB = HashMap<BookDir, Books>;
pub(crate) struct BooksFromDisk(HashMap<BookDir, HashSet<BookPath>>);

impl BooksFromDisk {
  pub(crate) fn new() -> Self {
    let map: HashMap<BookDir, HashSet<BookPath>> = Default::default();
    Self(map)
  }
  pub(crate) fn insert_book_path(&mut self, book_path: BookPath) {
    match self.get_mut(&book_path.parent_dir) {
      Some(set) => {
        match set.contains(&book_path) {
          true => {}
          false => {
            set.insert(book_path);
          }
        };
      }
      None => {
        let mut set: HashSet<BookPath> = Default::default();
        match set.contains(&book_path) {
          true => {}
          false => {
            set.insert(book_path.clone());
          }
        };
        self.insert(book_path.parent_dir, set);
      }
    };
  }
}
impl Deref for BooksFromDisk {
  type Target = HashMap<BookDir, HashSet<BookPath>>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}
impl DerefMut for BooksFromDisk {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

pub(crate) type DBBooksCount = usize;

pub struct ScanService {
  pub status: WorkStatus,
  not_cached_books: NotCachedBooks,
  settings: SETTINGS,
  db: DB,
  _error_handler: ErrorHandler,
}

impl ScanService {
  pub(crate) fn new(not_cached_books: NotCachedBooks, settings: SETTINGS, db: DB, error_handler: ErrorHandler) -> Self {
    Self { status: WorkStatus::NotWorking, not_cached_books, settings, db, _error_handler: error_handler }
  }
  pub fn run(&mut self) -> Result<()> {
    match &self.status {
      WorkStatus::Working => {}
      WorkStatus::NotWorking => {
        match self.settings.read().path_to_scan.clone() {
          None => {}
          Some(path_to_scan) => {
            let not_cached_books = self.not_cached_books.clone();
            let settings = self.settings.clone();
            let db = self.db.clone();

            thread::spawn(move || {
              let rt = Builder::new_multi_thread().enable_all().build().unwrap();

              rt.block_on(async move {
                let start_time = std::time::Instant::now();
                let books_from_disk = Self::get_books_from_disk(&path_to_scan, &settings);

                match BooksLocation::classify(&db, books_from_disk.len()).unwrap() {
                  BooksLocation::Disk => {
                    for (_book_dir, set) in books_from_disk.0 {
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

                info!("Total time of executing scan_service: {:?}", start_time.elapsed());
              });
            });
            self.status = WorkStatus::Working;
          }
        };
      }
    };
    Ok(())
  }
  fn get_books_from_disk(path_to_scan: &String, settings: &SETTINGS) -> BooksFromDisk {
    let start_time = std::time::Instant::now();
    let mut books_from_disk: BooksFromDisk = BooksFromDisk::new();
    for entry in WalkDir::new(path_to_scan) {
      match entry {
        Ok(entry) => {
          if entry.file_type().is_file() {
            let path = entry.path();
            match path.extension() {
              Some(_) => {
                let file_path: BookPath = BookPath::new(path.to_path_buf());
                if settings.contains_ext(&file_path.ext) {
                  books_from_disk.insert_book_path(file_path);
                }
              }
              None => {}
            };
          }
        }
        Err(_) => {}
      }
    }
    info!("The total time of receiving books from the disk: {:?}", start_time.elapsed());
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
                books.remove_book(book.book_path.clone(), db).await?;
              }
            };
          }
        }
        None => {
          num_of_outdated_books += books.storage.len();
          books.remove_self(db)?;
        }
      };
    }
    info!("number of outdated books: {:?}", &num_of_outdated_books);
    Ok(())
  }
  async fn insert_new_books_to_db(db: &DB, books_from_disk: BooksFromDisk) -> anyhow::Result<()> {
    let mut books_from_db: (BooksFromDB, DBBooksCount) = Books::all(&db);
    let mut num_of_new_books: usize = 0;
    for (book_dir, disk_books) in books_from_disk.0 {
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
    info!("number of new books: {:?}", &num_of_new_books);
    Ok(())
  }
}
