use std::{panic, thread};
use tokio::runtime::Builder;

use anyhow::Result;
use jwalk::WalkDir;
use tracing::{error, info};

use crate::{
  db::{
    DB, DBType,
    models::books::{
      Books,
      book::{BookDir, BookPath},
    },
  },
  error_handler::{ErrorHandler, ErrorType},
  not_cached_books::NotCachedBooks,
  services::{WorkStatus, notify_service::fs_handlers},
  settings::SETTINGS,
  types::{HashMap, HashSet},
};

pub(crate) type BooksFromDB = HashMap<BookDir, Books>;
pub(crate) type BooksFromDisk = HashMap<BookDir, HashSet<BookPath>>;
pub(crate) type DBBooksCount = usize;

enum BooksLocation {
  Disk,
  DB(BooksFromDB),
  DiskAndDB(BooksFromDB),
  None,
}
impl BooksLocation {
  fn classify(db: &DB, disk_books_count: usize) -> anyhow::Result<Self> {
    Ok(match &db.db_type {
      DBType::InMemory(_) => {
        if disk_books_count > 0 {
          info!("number of books on disk: {:?}", &disk_books_count);
          info!("number of books in db: 0");
          info!("number of new books: {:?}", &disk_books_count);
          Self::Disk
        } else {
          info!("number of books on disk: 0");
          info!("number of books in db: 0");
          Self::None
        }
      }
      DBType::InFile(_) => {
        let (books_from_db, db_books_count) = Books::all(&db);

        if books_from_db.len() > 0 && disk_books_count > 0 {
          info!("number of books on disk: {:?}", &disk_books_count);
          info!("number of books in db: {:?}", &books_from_db.len());
          Self::DiskAndDB(books_from_db)
        } else if books_from_db.len() > 0 && disk_books_count == 0 {
          info!("number of books on disk: 0");
          info!("number of books in db: {:?}", &books_from_db.len());
          info!("number of outdated books: {:?}", &db_books_count);
          Self::DB(books_from_db)
        } else if books_from_db.len() == 0 && disk_books_count > 0 {
          info!("number of books on disk: {:?}", &disk_books_count);
          info!("number of books in db: 0");
          info!("number of new books: {:?}", &disk_books_count);
          Self::Disk
        } else if books_from_db.len() == 0 && disk_books_count == 0 {
          info!("number of books on disk: 0");
          info!("number of books in db: 0");
          Self::None
        } else {
          info!("number of books on disk: 0");
          info!("number of books in db: 0");
          Self::None
        }
      }
    })
  }
}

pub struct ScanService {
  pub status: WorkStatus,
  not_cached_books: NotCachedBooks,
  settings: SETTINGS,
  db: DB,
  error_handler: ErrorHandler,
}

impl ScanService {
  pub(crate) fn new(not_cached_books: NotCachedBooks, settings: SETTINGS, db: DB, error_handler: ErrorHandler) -> Self {
    Self { status: WorkStatus::NotWorking, not_cached_books, settings, db, error_handler }
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
            let error_handler = self.error_handler.clone();

            let _worker = thread::spawn(move || {
              let result = panic::catch_unwind(panic::AssertUnwindSafe(|| -> anyhow::Result<()> {
                let rt = Builder::new_multi_thread().enable_all().build()?;

                rt.block_on(async move {
                  let start_time = std::time::Instant::now();
                  let books_from_disk = Self::get_books_from_disk(&path_to_scan, &settings);

                  match BooksLocation::classify(&db, books_from_disk.len()).unwrap() {
                    BooksLocation::Disk => {
                      for (_book_dir, set) in books_from_disk {
                        for book_path in set {
                          fs_handlers::insert_book(book_path, &db, &settings, &not_cached_books).await.unwrap();
                        }
                      }
                    }
                    BooksLocation::DB(books_from_db) => {
                      for (_book_dir, books) in books_from_db {
                        books.remove_books_in_dir(&db).unwrap();
                      }
                    }
                    BooksLocation::DiskAndDB(books_from_db) => {
                      Self::remove_outdated_books_from_db(&db, &books_from_disk, books_from_db).await.unwrap();
                      Self::insert_new_books_to_db(&db, books_from_disk).await.unwrap();
                    }
                    BooksLocation::None => {}
                  };

                  info!("Total time of executing scan_service: {:?}", start_time.elapsed());
                  Ok(())
                })
              }));

              match result {
                Ok(res) => {
                  match res {
                    Ok(_) => {}
                    Err(err) => {
                      error!("{:?}", err);
                    }
                  };
                }
                Err(panic_err) => {
                  let panic_msg = if let Some(s) = panic_err.downcast_ref::<&str>() {
                    s.to_string()
                  } else if let Some(s) = panic_err.downcast_ref::<String>() {
                    s.clone()
                  } else {
                    "Unknown panic".to_string()
                  };
                  error_handler.report(panic_msg, ErrorType::Other);
                }
              };
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
    let mut books_from_disk: BooksFromDisk = Default::default();
    for entry in WalkDir::new(path_to_scan) {
      match entry {
        Ok(entry) => {
          if entry.file_type().is_file() {
            let path = entry.path();
            match path.extension() {
              Some(_) => {
                let file_path: BookPath = BookPath::new(path.to_path_buf());
                if settings.contains_ext(&file_path.ext) {
                  match books_from_disk.get_mut(&file_path.parent_dir) {
                    Some(set) => {
                      match set.contains(&file_path) {
                        true => {}
                        false => {
                          set.insert(file_path);
                        }
                      };
                    }
                    None => {
                      let mut set: HashSet<BookPath> = Default::default();
                      match set.contains(&file_path) {
                        true => {}
                        false => {
                          set.insert(file_path.clone());
                        }
                      };
                      books_from_disk.insert(file_path.parent_dir, set);
                    }
                  };
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
          books.remove_books_in_dir(db)?;
        }
      };
    }
    info!("number of outdated books: {:?}", &num_of_outdated_books);
    Ok(())
  }
  async fn insert_new_books_to_db(db: &DB, books_from_disk: BooksFromDisk) -> anyhow::Result<()> {
    let mut books_from_db: (BooksFromDB, DBBooksCount) = Books::all(&db);
    let mut num_of_new_books: usize = 0;
    for (book_dir, disk_books) in books_from_disk {
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
