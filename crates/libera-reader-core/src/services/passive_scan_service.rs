use crate::db::crud::insert;
use crate::db::models::{Book, BookDataPK, DataOfUnhashedBook};
use crate::services::State;
use crate::types::{BookPath, BookSize, HashMap, HashSet};
use crate::utils::MultiThreadTask;
use anyhow::Result;
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use std::path::PathBuf;
use std::thread;
use tracing::{debug, info};
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
type BooksGroupedBySize = HashMap<BookSize, Vec<PathBuf>>;

impl State {
  //noinspection RsUnwrap
  pub fn run_passive_scan(&mut self) -> Result<()> {
    match &self.settings.read().path_to_scan {
      None => {}
      Some(path_to_scan) => {
        let start_time = std::time::Instant::now();
        let books_on_disk = self.get_books_from_disk(path_to_scan);
        let books_in_db = Book::get_all_existing_books(&self.db)?;

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

            debug!("Number of new books: {:?}", new_books.len());
            debug!("Number of general_books: {:?}", general_books.len());
            debug!("Number of outdated books: {:?}", outdated_books.len());
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
    let mut res: BooksGroupedBySize = Default::default();
    for i in books {
      let book_size = get_file_size(&i)?;
      match res.get_mut(&book_size) {
        Some(data) => { data.push(i); }
        None => { res.insert(book_size, vec![i]); }
      }
    }
    Ok(res)
  }
  fn classify_books(&self, books_grouped_by_size: BooksGroupedBySize) -> (UniqueSizeBooks, BooksForHashing) {
    let mut unique_size_books: UniqueSizeBooks = vec![];
    let mut books_for_hashing: BooksForHashing = vec![];
    for (book_size, books) in books_grouped_by_size {
      if books.len() == 1 {
        unique_size_books.push((book_size, books[0].clone()));
      } else if books.len() > 1 {
        books_for_hashing.push((book_size, books[0].clone()));
      }
    }
    (unique_size_books, books_for_hashing)
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
    debug!("The total time of receiving books from the disk: {:?}", start_time.elapsed());
    books_from_disk
  }
  fn insert_new_books_to_db(&self, new_books: Vec<PathBuf>) -> Result<()> {
    let books_grouped_by_size = self.group_books_by_size(new_books)?;
    let (unique_size_books, books_for_hashing) = self.classify_books(books_grouped_by_size);
    debug!("Number of books for hashing: {:?}", books_for_hashing.len());
    debug!("Number of books of a unique size: {:?}", unique_size_books.len());
    for (book_size, book_pathbuf) in unique_size_books {
      let book_data_pk = BookDataPK::UniqueSize(book_size.clone());
      let new_book = Book::from_pathbuf(&book_pathbuf, book_data_pk);
      let data = DataOfUnhashedBook::new(book_size.clone(), vec![new_book.full_path.clone()]);
      insert::<DataOfUnhashedBook>(data, &self.db)?;
      insert::<Book>(new_book, &self.db)?;
    }
    let db = self.db.clone();
    thread::spawn(move || {
      let num_of_threads = MultiThreadTask::CalcHash.get_num_of_threads();
      debug!("Number of threads for hash calculation: {:?}", &num_of_threads);
      ThreadPoolBuilder::new().num_threads(num_of_threads).build().unwrap().install(|| {
        books_for_hashing.into_par_iter().for_each(|(book_size, book_pathbuf)| {
          Book::insert_to_db(&book_pathbuf, book_size, &db).unwrap()
        });
      });
    });
    Ok(())
  }
  fn remove_outdated_books_from_db(&self, books: Vec<Book>) -> Result<()> {
    for book in books {
      Book::remove(book, &self.db)?
    }
    Ok(())
  }
}
