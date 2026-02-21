pub mod book;
pub(crate) mod book_hashes;
pub(crate) mod book_sizes;
pub mod bookmark;
pub(crate) mod mutool_data;
pub(crate) mod thumbnail;
pub(crate) mod user_data;
use std::hash::{Hash, Hasher};
use utils::debug;

use crate::{
  db::{
    DB,
    models::{
      MutoolData,
      books::{
        book::{Book, BookDir, BookName, BookPath},
        book_sizes::BookSizes,
      },
    },
  },
  services::scan_service::{BooksFromDB, DBBooksCount},
  types::HashMap,
};

use native_db::*;
#[allow(unused_imports)]
use native_model::{Model, native_model};
use serde::{Deserialize, Serialize};

pub(crate) type BookHash = String;

impl PartialEq for Books {
  fn eq(&self, other: &Self) -> bool {
    self.parent_dir.eq(&other.parent_dir)
  }
}
impl Hash for Books {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.parent_dir.hash(state);
  }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum DuplicateBookData {
  BookHash(BookHash),
  MutoolData(Option<MutoolData>),
}
#[derive(Serialize, Deserialize, Clone)]
pub enum BookType {
  UniqueSize { book_path: BookPath, mutool_data: Option<MutoolData> },
  DuplicateSize(HashMap<BookPath, DuplicateBookData>),
}

#[derive(Serialize, Deserialize, Clone, Eq, Debug)]
#[native_model(id = 0, version = 1)]
#[native_db]
pub struct Books {
  #[primary_key]
  pub parent_dir: BookDir,
  pub(crate) storage: HashMap<BookName, Book>,
}
impl Books {
  fn new(book: Book) -> Self {
    let parent_dir = book.book_path.parent_dir.clone();
    let mut storage = HashMap::default();
    storage.insert(book.book_path.name.clone(), book);
    Self { parent_dir, storage }
  }
  pub fn get_by_path(book_path: BookPath, db: &DB) -> anyhow::Result<Option<Book>> {
    match Books::get_by_parent_dir(book_path.parent_dir, db)? {
      Some(books) => match books.storage.get(&book_path.name) {
        Some(book) => Ok(Some(book.clone())),
        None => Ok(None),
      },
      None => Ok(None),
    }
  }
  pub fn get_by_parent_dir(parent_dir: BookDir, db: &DB) -> anyhow::Result<Option<Self>> {
    db.get_primary::<Books>(parent_dir)
  }
  pub(crate) async fn insert_many(&mut self, books: impl IntoIterator<Item = BookPath>, db: &DB) -> anyhow::Result<()> {
    let old_self = self.clone();
    for book_path in books {
      let book_name = book_path.name.clone();
      let new_book = Book::new(book_path).await?;
      self.storage.insert(book_name, new_book);
    }
    db.update(old_self, self.clone())?;
    Ok(())
  }
  pub(crate) async fn insert_many_and_create_new_self(parent_dir: BookDir, books: impl IntoIterator<Item = BookPath>, db: &DB) -> anyhow::Result<()> {
    let mut storage = HashMap::default();
    for book_path in books {
      let book = Book::new(book_path).await?;
      storage.insert(book.book_path.name.clone(), book);
    }
    db.insert::<Self>(Self { parent_dir, storage })?;
    Ok(())
  }
  pub(crate) async fn insert_book(new_book: Book, db: &DB) -> anyhow::Result<()> {
    let parent_dir = new_book.book_path.parent_dir.clone();
    match Books::get_by_parent_dir(parent_dir, db).unwrap() {
      Some(old_books) => {
        let mut updated_books = old_books.clone();
        match updated_books.storage.get_mut(&new_book.book_path.name) {
          Some(book) => {
            if book.book_path.deleted {
              book.book_path.deleted = false;
              db.update::<Self>(old_books, updated_books).unwrap(); // error
            }
          }
          None => {
            updated_books.storage.insert(new_book.book_path.name.clone(), new_book);
            db.update::<Self>(old_books, updated_books).unwrap(); // error
          }
        };
      }
      None => {
        db.insert::<Self>(Self::new(new_book)).unwrap();
      }
    };
    Ok(())
  }
  pub fn all(db: &DB) -> (BooksFromDB, DBBooksCount) {
    let mut db_books_count: usize = 0;
    let mut res: HashMap<BookDir, Books> = Default::default();
    for books in db.scan_primary::<Self>().unwrap() {
      db_books_count += books.storage.len();
      res.insert(books.parent_dir.clone(), books);
    }
    (res, db_books_count)
  }
  pub(crate) fn remove_self(self, db: &DB) -> anyhow::Result<()> {
    let start_time = std::time::Instant::now();
    match self.storage.is_empty() {
      true => {
        db.remove(self).unwrap();
      }
      false => {
        let mut updated_books = self.clone();
        let mut deleted_books: Vec<Book> = vec![];
        let mut new_storage: HashMap<BookName, Book> = Default::default();

        for (book_name, mut book) in updated_books.storage {
          match book.can_delete() {
            true => {
              deleted_books.push(book);
            }
            false => {
              book.mark_as_deleted();
              BookSizes::mark_book_path_as_deleted(book.book_size, &book.book_path, db).unwrap();
              new_storage.insert(book_name, book);
            }
          };
        }
        updated_books.storage = new_storage;

        for book in deleted_books {
          BookSizes::remove_book(book.book_size, &book.book_path, db).unwrap();
        }
        let dir_exsits = updated_books.parent_dir.exists();
        let storage_is_empty = updated_books.storage.is_empty();
        match storage_is_empty || !dir_exsits {
          true => {
            db.remove(self).unwrap();
          }
          false => {
            db.update(self, updated_books).unwrap();
          }
        };
      }
    };
    let total_time = start_time.elapsed();
    debug!("The total time of deleting books in the dir: {:?}", &total_time);
    Ok(())
  }
  pub(crate) async fn remove_book(&self, book_path: BookPath, db: &DB) -> anyhow::Result<()> {
    let old_self = self.clone();
    let mut updated_self = self.clone();
    if let Some(outdated_book_link) = updated_self.storage.get_mut(&book_path.name) {
      let start_time = std::time::Instant::now();
      match outdated_book_link.can_delete() {
        true => {
          if let Some(outdated_book) = updated_self.storage.swap_remove(&book_path.name) {
            BookSizes::remove_book(outdated_book.book_size, &book_path, db)?;
          };
        }
        false => {
          outdated_book_link.mark_as_deleted();
          BookSizes::mark_book_path_as_deleted(outdated_book_link.book_size, &outdated_book_link.book_path, db)?;
        }
      };
      db.update(old_self, updated_self)?;
      let total_time = start_time.elapsed();
      debug!("The total time of deleting a book: {:?}", &total_time);
    };
    Ok(())
  }
}
