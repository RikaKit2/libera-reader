use crate::db::models::books::Books;
use crate::db::models::books::book::Book;
use crate::services::Services;
use crate::types::HashMap;
use anyhow::Result;
use jwalk::WalkDir;
use std::path::PathBuf;
use tracing::info;

pub(crate) enum BooksLocation {
  Disk,
  DB,
  DiskAndDB,
  None,
}

impl Services {
  pub async fn run_passive_scan(&mut self) -> Result<()> {
    match &self.settings.read().path_to_scan {
      None => {}
      Some(path_to_scan) => {
        let start_time = std::time::Instant::now();
        let books_on_disk = self.get_books_from_disk(path_to_scan);
        let books_from_db = Books::all(&self.db)?;

        let num_books_on_disk = books_on_disk.len();
        let mut num_books_from_db: usize = 0;

        let mut actual_books: Vec<Box<Books>> = Default::default();

        for mut books in books_from_db {
          match &mut books.books_type {
            Type::Unique(unique_book) => {
              match books_on_disk.contains_key(&unique_book.full_path()) {
                true => {
                  actual_books.push(Box::new(books));
                }
                false => {}
              };
            }
            Type::Кepeating(books_with_common_dir) => {
              for (parent_dir, books_map) in books_with_common_dir.iter_mut() {
                let target_hash = None;
                for (books_hash, hashed_books) in books_map.iter_mut() {
                  let mut outdated_books: Vec<Box<Book>> = vec![];
                  for book in hashed_books.books.iter() {
                    let book_path = book.full_path_str(&parent_dir);
                    match books_on_disk.contains_key(&book_path) {
                      true => {}
                      false => {
                        outdated_books.push(Box::new(book.clone()));
                      }
                    };
                  }
                  for i in outdated_books {
                    hashed_books.books.remove(&i);
                  }
                }
                match books_map.get(&books_hash) {
                  Some(hashed_books) if hashed_books.books.is_empty() => {
                    books_map.remove(&books_hash);
                  }
                  None => {}
                }
              }
            }
          };
        }
      }
    };
    Ok(())
  }
  pub(crate) fn get_books_from_disk(&self, path_to_scan: &String) -> HashMap<BookPath, Box<PathBuf>> {
    let start_time = std::time::Instant::now();
    let mut books_from_disk: HashMap<BookPath, Box<PathBuf>> = Default::default();
    for entry in WalkDir::new(path_to_scan) {
      match entry {
        Ok(entry) => {
          if entry.file_type().is_file() {
            let path = entry.path();
            match path.extension() {
              Some(res) => {
                let file_ext = res.to_str().unwrap();
                if self.settings.contains_ext(file_ext) {
                  let buf = path.to_path_buf();
                  let book_path = buf.to_str().unwrap().to_string();
                  books_from_disk.insert(book_path, Box::new(buf));
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
