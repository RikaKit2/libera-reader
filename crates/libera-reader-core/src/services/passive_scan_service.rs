use crate::db::DB;
use crate::db::models::{Book, DataOfUnhashedBook};
use crate::services::Services;
use crate::types::{BookHash, BookPath, BookSize, HashMap, HashSet, NotCachedBooks};
use anyhow::Result;
use dashmap::DashMap;
use gxhash::GxBuildHasher;
use jwalk::WalkDir;
use std::path::PathBuf;
use tracing::{error, info};
use utils::{calc_file_hash, get_file_size};

pub(crate) enum BooksLocation {
  Disk,
  DB,
  DiskAndDB,
  None,
}
type UniqueSizeBooks = Vec<(BookSize, PathBuf)>;
type BooksForHashing = Vec<(BookSize, PathBuf)>;
pub(crate) type BooksGroupedBySize = DashMap<BookSize, Vec<PathBuf>, GxBuildHasher>;

impl Services {
  //noinspection RsUnwrap
  pub async fn run_passive_scan(&mut self) -> Result<()> {
    match &self.settings.read().path_to_scan {
      None => {}
      Some(path_to_scan) => {
        let start_time = std::time::Instant::now();
        let books_on_disk = self.get_books_from_disk(path_to_scan);
        let books_in_db = Book::get_all_existing_books(&self.db)?;
        info!("Number of books on disk: {:?}", books_on_disk.len());

        match self.get_books_location(books_in_db.len(), books_on_disk.len()) {
          BooksLocation::Disk => {
            if books_on_disk.len() > 0 {
              let (unique_size_books, books_for_hashing) = self.classify_books(books_on_disk).await?;
              let db = self.db.clone();
              let not_cached_books = self.not_cached_books.clone();

              if books_for_hashing.len() > 0 {
                tokio::spawn(async move {
                  for (book_size, book_pathbuf) in books_for_hashing {
                    let book_hash: BookHash = calc_file_hash(&book_pathbuf).await.unwrap();
                    Book::insert_book_to_hashed(&book_pathbuf, book_size, book_hash, &db, &not_cached_books).unwrap();
                  }
                });
              }

              if unique_size_books.len() > 0 {
                for (book_size, book_pathbuf) in unique_size_books {
                  Book::insert_book_of_unique_size(&book_pathbuf, book_size, &self.db, &self.not_cached_books);
                }
              }
            }
          }
          BooksLocation::DB => {
            self.remove_outdated_books(books_in_db)?;
          }
          BooksLocation::DiskAndDB => {
            let mut books_on_disk: HashMap<BookPath, PathBuf> = books_on_disk.into_iter().map(|i| (i.to_str().unwrap().to_string(), i)).collect();
            let mut books_in_db: HashMap<BookPath, Book> = books_in_db.into_iter().map(|i| (i.full_path.clone(), i)).collect();

            let books_paths_on_disk: HashSet<BookPath> = books_on_disk.keys().cloned().collect();
            let books_paths_in_db: HashSet<BookPath> = books_in_db.keys().cloned().collect();

            let new_books: Vec<PathBuf> = books_paths_on_disk.difference(&books_paths_in_db).map(|i| books_on_disk.remove(i).unwrap()).collect();
            let general_books: Vec<Book> = books_paths_on_disk.intersection(&books_paths_in_db).map(|i| books_in_db.remove(i).unwrap()).collect();
            let outdated_books: Vec<Book> =
              books_paths_in_db.difference(&books_paths_on_disk).map(|book_path| books_in_db.remove(book_path).unwrap()).collect();

            info!("Number of general_books: {:?}", general_books.len());
            info!("Number of outdated books: {:?}", outdated_books.len());
            self.cache_general_books(general_books)?;
            self.remove_outdated_books(outdated_books)?;
            self.insert_new_books(new_books).await?;
          }
          BooksLocation::None => {}
        };
        info!("Dir scan service execution time is: {:?}", start_time.elapsed());
      }
    }
    Ok(())
  }
  pub(crate) fn get_books_location(&self, db_book_count: usize, disk_book_count: usize) -> BooksLocation {
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
  pub(crate) async fn group_books_by_size(&self, books: Vec<PathBuf>) -> Result<BooksGroupedBySize> {
    let res: BooksGroupedBySize = Default::default();
    for i in books {
      let book_size = get_file_size(&i).await?;
      match res.get_mut(&book_size) {
        Some(mut data) => {
          data.push(i);
        }
        None => {
          res.insert(book_size, vec![i]);
        }
      }
    }
    Ok(res)
  }
  async fn classify_books_using_db(&self, books: Vec<PathBuf>, db: &DB) -> Result<(UniqueSizeBooks, BooksForHashing)> {
    info!("Number of new books: {:?}", books.len());
    let start_time = std::time::Instant::now();
    let mut unique_size_books: UniqueSizeBooks = vec![];
    let mut books_for_hashing: BooksForHashing = vec![];
    let unique_size_books_in_db: HashMap<BookSize, DataOfUnhashedBook> =
      db.scan_primary::<DataOfUnhashedBook>()?.into_iter().map(|i| (i.book_size.clone(), i)).collect();

    for (book_size, books) in self.group_books_by_size(books).await? {
      if books.len() == 1 {
        match unique_size_books_in_db.contains_key(&book_size) {
          true => unique_size_books.push((book_size, books[0].clone())),
          false => books_for_hashing.push((book_size, books[0].clone())),
        };
      } else if books.len() > 1 {
      }
    }

    info!("Total time of classify_books: {:?}", start_time.elapsed());
    info!("Number of new books for hashing: {:?}", books_for_hashing.len());
    info!("Number of new unique size books: {:?}", unique_size_books.len());
    Ok((unique_size_books, books_for_hashing))
  }
  async fn classify_books(&self, books: Vec<PathBuf>) -> Result<(UniqueSizeBooks, BooksForHashing)> {
    info!("Number of new books: {:?}", books.len());
    let start_time = std::time::Instant::now();
    let mut unique_size_books: UniqueSizeBooks = vec![];
    let mut books_for_hashing: BooksForHashing = vec![];
    for (book_size, books) in self.group_books_by_size(books).await? {
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
  pub(crate) fn get_books_from_disk(&self, path_to_scan: &String) -> Vec<PathBuf> {
    let start_time = std::time::Instant::now();
    let mut books_from_disk: Vec<PathBuf> = vec![];
    for entry in WalkDir::new(path_to_scan) {
      match entry {
        Ok(entry) => {
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
        }
        Err(_) => {}
      }
    }
    info!("The total time of receiving books from the disk: {:?}", start_time.elapsed());
    books_from_disk
  }
  async fn insert_new_books(&self, new_books: Vec<PathBuf>) -> Result<()> {
    if new_books.len() > 0 {
      let (unique_size_books, books_for_hashing) = self.classify_books(new_books).await?;
      self.insert_books_for_hashing(books_for_hashing);
      self.insert_unique_books(unique_size_books).await?;
    }
    Ok(())
  }
  fn remove_outdated_books(&self, books: Vec<Book>) -> Result<()> {
    for i in books {
      i.remove(&self.db)?
    }
    Ok(())
  }
  async fn insert_unique_books(&self, unique_size_books: UniqueSizeBooks) -> Result<()> {
    if unique_size_books.len() > 0 {
      let start_time = std::time::Instant::now();
      for (book_size, book_pathbuf) in unique_size_books {
        Book::insert(&book_pathbuf, book_size, &self.db, &self.not_cached_books).await?;
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
        for (book_size, book_pathbuf) in books_for_hashing {
          Book::insert(&book_pathbuf, book_size, &db, &not_cached_books).await.unwrap();
        }
      });
    }
  }
  fn cache_general_books(&self, books: Vec<Book>) -> Result<()> {
    if books.len() > 0 {
      let db = self.db.clone();
      let not_cached_books = self.not_cached_books.clone();
      let task = tokio::spawn(async move {
        for book in books {
          let book_data = book.get_book_data(&db).unwrap();
          match book_data {
            None => {
              error!("book_data is none")
            }
            Some(book_data) => match book_data.cached == false && book_data.mutool_err.is_none() {
              true => {
                not_cached_books.push(Box::new(book.to_pathbuf())).unwrap();
              }
              false => {}
            },
          }
        }
        not_cached_books
      });
      tokio::spawn(async move {
        let start_time = std::time::Instant::now();
        let not_cached_books: NotCachedBooks = task.await.unwrap();
        info!("Total time of adding general books to not_cached_books: {:?}", start_time.elapsed());
        info!("Number of not_cached_books: {:?}", not_cached_books.len());
      });
    }
    Ok(())
  }
}
