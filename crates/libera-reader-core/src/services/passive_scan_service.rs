use crate::db::models::{HashedBooks, UniqueBook};
use crate::services::Services;
use crate::types::{BookPath, BookSize, BookType, HashMap, HashSet};
use anyhow::Result;
use dashmap::DashMap;
use gxhash::GxBuildHasher;
use jwalk::WalkDir;
use std::path::PathBuf;
use tracing::info;
use utils::get_file_size;

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
  pub async fn run_passive_scan(&mut self) -> Result<()> {
    match &self.settings.read().path_to_scan {
      None => {}
      Some(path_to_scan) => {
        let start_time = std::time::Instant::now();
        let books_on_disk = self.get_books_from_disk(path_to_scan);

        let mut unique_db_books: HashMap<BookPath, UniqueBook> = UniqueBook::get_all(&self.db)?.into_iter().map(|i| (i.full_path.clone(), i)).collect();
        let hashed_books: Vec<HashedBooks> = self.db.scan_primary::<HashedBooks>()?;

        let db_books_count = hashed_books.len() + unique_db_books.len();
        info!("Number of books on disk: {:?}", books_on_disk.len());

        match self.get_books_location(db_books_count, books_on_disk.len()) {
          BooksLocation::Disk => {
            self.insert_new_books(books_on_disk).await;
          }
          BooksLocation::DB => {
            for (_, book) in unique_db_books.into_iter() {
              book.remove(&self.db)?;
            }
            for i in hashed_books {
              i.remove(&self.db)?;
            }
          }
          BooksLocation::DiskAndDB => {
            let mut books_on_disk: HashMap<BookPath, PathBuf> = books_on_disk.into_iter().map(|i| (i.to_str().unwrap().to_string(), i)).collect();
            let books_paths_on_disk: HashSet<BookPath> = books_on_disk.keys().cloned().collect();

            let mut hashed_books_for_caching: Vec<HashedBooks> = vec![];
            let mut unique_books_for_caching: Vec<UniqueBook> = vec![];

            let mut existing_hashed_books_paths: Vec<BookPath> = vec![];
            let mut existing_unique_books_paths: HashSet<BookPath> = Default::default();

            let mut outdated_unique_books: Vec<UniqueBook> = vec![];
            let mut outdated_hashed_books: Vec<HashedBooks> = vec![];

            for (path, book) in unique_db_books {
              match books_paths_on_disk.contains(&path) {
                true => {
                  existing_unique_books_paths.insert(path);
                  unique_books_for_caching.push(book);
                }
                false => outdated_unique_books.push(book),
              }
            }

            for i in hashed_books {
              let num_of_paths = i.books.len();
              let mut outdated_books_count: usize = 0;

              for (path, _) in i.books.iter() {
                match books_paths_on_disk.contains(path) {
                  true => {}
                  false => {
                    outdated_books_count += 1;
                  }
                }
              }

              if outdated_books_count == num_of_paths {
                outdated_hashed_books.push(i);
              } else {
                existing_hashed_books_paths.extend(i.books.keys().cloned());
                if i.mutool_data.is_cached() == false {
                  hashed_books_for_caching.push(i);
                }
              }
            }

            let mut existing_books_in_db = existing_unique_books_paths;
            existing_books_in_db.extend(existing_hashed_books_paths);
          }
          BooksLocation::None => {}
        }
      }
    };
    Ok(())
  }
  pub(crate) async fn classify_books(
    &self, existing_hashed_books_paths: HashMap<BookSize, Vec<BookPath>>, existing_unique_books_paths: HashMap<BookSize, BookPath>, books_on_disk: Vec<PathBuf>,
  ) -> Result<(UniqueSizeBooks, BooksForHashing)> {
    let start_time = std::time::Instant::now();
    let mut unique_size_books: UniqueSizeBooks = vec![];
    let mut books_for_hashing: BooksForHashing = vec![];

    for (book_size, books) in self.group_books_by_size(books_on_disk).await? {
      match existing_hashed_books_paths.get(&book_size) {
        Some(books) => {}
        None => match existing_unique_books_paths.get(&book_size) {
          Some(book_path) => {
            // take previous book and move to hashed
            for buf in books {
              books_for_hashing.push((book_size.clone(), buf));
            }
          }
          None => {
            if books.len() == 1 {
              unique_size_books.push((book_size, books[0].clone()));
            } else if books.len() > 1 {
              for book in books {
                books_for_hashing.push((book_size.clone(), book));
              }
            }
          }
        },
      }
    }
    info!("Total time of classify_books: {:?}", start_time.elapsed());
    info!("Number of new books for hashing: {:?}", books_for_hashing.len());
    info!("Number of new unique size books: {:?}", unique_size_books.len());
    Ok((unique_size_books, books_for_hashing))
  }
  async fn insert_new_books(&self, new_books: Vec<PathBuf>) -> anyhow::Result<()> {
    // let (unique_books, books_for_hashing) = self.classify_books(new_books).await?;
    // self.insert_unique_books(unique_books).await?;
    // self.insert_books_for_hashing(books_for_hashing);
    Ok(())
  }
  pub(crate) async fn insert_books_for_hashing(&self, books_for_hashing: BooksForHashing) -> anyhow::Result<()> {
    if books_for_hashing.len() > 0 {
      let db = self.db.clone();
      let not_cached_books = self.not_cached_books.clone();
      tokio::spawn(async move {
        for (book_size, book_pathbuf) in books_for_hashing {
          HashedBooks::insert(&book_pathbuf, &db, book_size, &not_cached_books).await.unwrap();
        }
      });
    }
    Ok(())
  }
  pub(crate) async fn insert_unique_books(&self, unique_books: Vec<(BookSize, PathBuf)>) -> anyhow::Result<()> {
    if unique_books.len() > 0 {
      let start_time = std::time::Instant::now();
      for (book_size, book_path) in unique_books {
        UniqueBook::insert(&book_path, book_size, &self.db, &self.not_cached_books).await?;
      }
      info!("Total time of adding unique books: {:?}", start_time.elapsed());
    }
    Ok(())
  }
  fn remove_hashed_books(&self, hashed_books: Vec<HashedBooks>, books_on_disk: &HashMap<BookPath, PathBuf>) -> anyhow::Result<Vec<BookPath>> {
    let mut general_hashed_books: Vec<BookPath> = vec![];

    if !hashed_books.is_empty() {
      for mut i in hashed_books {
        let old_book = i.clone();

        for path in old_book.books.keys() {
          match books_on_disk.contains_key(path) {
            true => {
              general_hashed_books.push(path.clone());
            }
            false => match i.books.get(path) {
              Some(_) => {
                i.books.remove(path).unwrap();
              }
              None => {}
            },
          };
        }

        if i.books.is_empty() {
          i.remove(&self.db)?;
        } else {
          match i.mutool_data.is_cached() {
            true => {}
            false => {
              self.not_cached_books.push(Box::new(BookType::Hashed(i.book_hash)));
            }
          }
        };
      }
    }
    Ok(general_hashed_books)
  }
  //noinspection RsUnwrap
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
}
