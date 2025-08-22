use crate::db::models::Book;
use crate::services::Services;
use crate::types::{BookPath, BookSize, HashMap, HashSet};
use anyhow::Result;
use dashmap::DashMap;
use gxhash::GxBuildHasher;
use rayon::prelude::*;
use std::path::PathBuf;
use tracing::{error, info};
use utils::get_file_size;
use walkdir::WalkDir;

enum BooksLocation {
  Disk,
  DB,
  DiskAndDB,
  None,
}
type UniqueSizeBooks = Vec<(BookSize, PathBuf)>;
type BooksForHashing = Vec<(BookSize, PathBuf)>;
type BooksGroupedBySize = DashMap<BookSize, Vec<PathBuf>, GxBuildHasher>;

impl Services {
  //noinspection RsUnwrap
  pub fn run_passive_scan(&mut self) -> Result<()> {
    match &self.settings.read().path_to_scan {
      None => {}
      Some(path_to_scan) => {
        let start_time = std::time::Instant::now();
        let books_on_disk = self.get_books_from_disk(path_to_scan);
        let books_in_db = Book::get_all_existing_books(&self.db)?;
        info!("Number of books on disk: {:?}", books_on_disk.len());

        match self.get_books_location(books_in_db.len(), books_on_disk.len()) {
          BooksLocation::Disk => {
            self.insert_new_books_to_db(books_on_disk)?;
          }
          BooksLocation::DB => {
            self.remove_outdated_books_from_db(books_in_db)?;
          }
          BooksLocation::DiskAndDB => {
            let mut books_on_disk: HashMap<BookPath, PathBuf> = books_on_disk.into_iter().map(|i| (i.to_str().unwrap().to_string(), i)).collect();
            let mut books_in_db: HashMap<BookPath, Book> = books_in_db.into_iter().map(|i| (i.full_path.clone(), i)).collect();

            let books_paths_on_disk: HashSet<BookPath> = books_on_disk.keys().cloned().collect();
            let books_paths_in_db: HashSet<BookPath> = books_in_db.keys().cloned().collect();

            let new_books: Vec<PathBuf> = books_paths_on_disk.difference(&books_paths_in_db).map(|i| {
              books_on_disk.remove(i).unwrap()
            }).collect();
            let general_books: Vec<Book> = books_paths_on_disk.intersection(&books_paths_in_db).map(|i| {
              books_in_db.remove(i).unwrap()
            }).collect();
            let outdated_books: Vec<Book> = books_paths_in_db.difference(&books_paths_on_disk).map(|book_path|
              books_in_db.remove(book_path).unwrap()
            ).collect();

            info!("Number of general_books: {:?}", general_books.len());
            info!("Number of outdated books: {:?}", outdated_books.len());
            self.cache_general_books(general_books)?;
            self.remove_outdated_books_from_db(outdated_books)?;
            self.insert_new_books_to_db(new_books)?;
          }
          BooksLocation::None => {}
        };
        info!("Dir scan service execution time is: {:?}", start_time.elapsed());
      }
    }
    Ok(())
  }
  fn get_books_location(&self, db_book_count: usize, disk_book_count: usize) -> BooksLocation {
    if db_book_count > 0 && disk_book_count == 0 {
      BooksLocation::DB
    } else if db_book_count == 0 && disk_book_count > 0 {
      BooksLocation::Disk
    } else if db_book_count > 0 && disk_book_count > 0 {
      BooksLocation::DiskAndDB
    } else {
      BooksLocation::None
    }
  }
  fn group_books_by_size(&self, books: Vec<PathBuf>) -> Result<BooksGroupedBySize> {
    let res: BooksGroupedBySize = Default::default();
    books.into_par_iter().for_each(|i| {
      let book_size = get_file_size(&i).unwrap();
      match res.get_mut(&book_size) {
        Some(mut data) => { data.push(i); }
        None => { res.insert(book_size, vec![i]); }
      }
    });
    Ok(res)
  }
  fn classify_books(&self, books: Vec<PathBuf>) -> Result<(UniqueSizeBooks, BooksForHashing)> {
    let start_time = std::time::Instant::now();
    let mut unique_size_books: UniqueSizeBooks = vec![];
    let mut books_for_hashing: BooksForHashing = vec![];
    for (book_size, books) in self.group_books_by_size(books)? {
      if books.len() == 1 {
        unique_size_books.push((book_size, books[0].clone()));
      } else if books.len() > 1 {
        for book in books {
          books_for_hashing.push((book_size.clone(), book));
        }
      }
    }
    info!("Total time of classify_books: {:?}", start_time.elapsed());
    info!("Number of new books for hashing: {:?}", books_for_hashing.len());
    info!("Number of new unique size books: {:?}", unique_size_books.len());
    Ok((unique_size_books, books_for_hashing))
  }
  fn get_books_from_disk(&self, path_to_scan: &String) -> Vec<PathBuf> {
    let start_time = std::time::Instant::now();
    let mut books_from_disk: Vec<PathBuf> = vec![];
    for entry in WalkDir::new(path_to_scan) {
      let entry = entry.unwrap();
      if entry.file_type().is_file() {
        let path = entry.path();
        match path.extension() {
          Some(res) => {
            let file_ext = res.to_str().unwrap();
            if self.settings.contains_ext(file_ext) {
              books_from_disk.push(path.to_path_buf());
            }
          }
          None => {}
        };
      }
    };
    info!("The total time of receiving books from the disk: {:?}", start_time.elapsed());
    books_from_disk
  }
  fn insert_new_books_to_db(&self, new_books: Vec<PathBuf>) -> Result<()> {
    info!("Number of new books: {:?}", new_books.len());
    if new_books.len() > 0 {
      let (unique_size_books, books_for_hashing) = self.classify_books(new_books)?;
      self.insert_books_for_hashing(books_for_hashing);
      self.insert_unique_books(unique_size_books)?;
    }
    Ok(())
  }
  fn remove_outdated_books_from_db(&self, books: Vec<Book>) -> Result<()> {
    books.into_par_iter().for_each(|i| Book::remove(i, &self.db).unwrap());
    Ok(())
  }
  fn insert_unique_books(&self, unique_size_books: UniqueSizeBooks) -> Result<()> {
    if unique_size_books.len() > 0 {
      let start_time = std::time::Instant::now();
      for (book_size, book_pathbuf) in unique_size_books {
        Book::insert_to_books_of_unique_size(&book_pathbuf, book_size, &self.db, &self.not_cached_books);
      }
      info!("Total time of adding unique books: {:?}", start_time.elapsed());
    }
    Ok(())
  }
  fn insert_books_for_hashing(&self, books_for_hashing: BooksForHashing) {
    if books_for_hashing.len() > 0 {
      let db = self.db.clone();
      let not_cached_books = self.not_cached_books.clone();
      tokio::spawn(async move {
        let start_time = std::time::Instant::now();
        rayon::ThreadPoolBuilder::new().num_threads(num_cpus::get()).build().unwrap().install(|| {
          books_for_hashing.into_par_iter().for_each(|(book_size, book_pathbuf)| {
            Book::insert_to_hashed_books(&book_pathbuf, book_size, &db, &not_cached_books);
          });
        });
        info!("Total time of hashing books: {:?}", start_time.elapsed());
      });
    }
  }
  fn cache_general_books(&self, books: Vec<Book>) -> Result<()> {
    if books.len() > 1 {
      let db = self.db.clone();
      let not_cached_books = self.not_cached_books.clone();
      tokio::spawn(async move {
        let start_time = std::time::Instant::now();
        for book in books {
          let book_data = book.get_book_data(&db).unwrap();
          match book_data {
            None => { error!("book_data is none") }
            Some(book_data) => {
              match book_data.cached == false && book_data.mutool_err.is_none() {
                true => { not_cached_books.push(Box::new(book.to_pathbuf())).unwrap(); }
                false => {}
              }
            }
          }
        }
        if not_cached_books.len() > 0 {
          info!("Total time of adding general books to not_cached_books: {:?}", start_time.elapsed());
        }
        info!("Number of not_cached_books: {:?}", not_cached_books.len());
      });
    }
    Ok(())
  }
}
