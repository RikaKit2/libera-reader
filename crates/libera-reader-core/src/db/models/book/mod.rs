mod insert_to_db;
mod delete;

use crate::db::models::book_data::BookData;
use crate::db::models::book_data_pk::BookDataPK;
use crate::db::models::{DataOfHashedBook, DataOfUnhashedBook};
use crate::types::{BookPath, BookSize, APP_DIRS, DB};
use anyhow::Result;
use mutool_bindings::MuToolResult;
use native_db::*;
#[allow(unused_imports)]
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use tracing::info;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[native_model(id = 5, version = 1)]
#[native_db]
pub struct Book {
  #[primary_key]
  pub full_path: String,
  pub path_to_dir: String,
  pub dir_name: String,
  pub book_name: String,
  pub ext: String,
  pub book_data_pk: BookDataPK,
}
impl Book {
  pub(crate) fn from_pathbuf(future_book: &PathBuf, book_data: BookDataPK) -> Self {
    Self {
      full_path: future_book.to_str().unwrap().to_string(),
      path_to_dir: future_book.parent().unwrap().to_str().unwrap().to_string(),
      book_name: future_book.file_name().unwrap().to_str().unwrap().to_string(),
      dir_name: future_book.parent().unwrap().file_name().unwrap().to_str().unwrap().to_string(),
      ext: future_book.extension().unwrap().to_str().unwrap().to_string(),
      book_data_pk: book_data,
    }
  }
  pub(crate) fn mark_as_cached(self, db: &DB) -> Result<()> {
    self.book_data_pk.update(|book_data: &mut BookData| {
      book_data.thumbnail = Some(vec![]);
    }, db)
  }
  pub(crate) fn mark_as_broken(self, mutool_err: MuToolResult, db: &DB) -> Result<()> {
    self.book_data_pk.update(|book_data: &mut BookData| {
      book_data.mutool_err = Some(mutool_err);
    }, db)
  }
  pub(crate) fn path_to_thumbnail(&self, app_dirs: &APP_DIRS) -> PathBuf {
    match &self.book_data_pk {
      BookDataPK::UniqueSize(book_size) => app_dirs.read().dir_of_unhashed_books.join(book_size.to_string()).with_extension("png"),
      BookDataPK::RepeatingSize(book_hash) => app_dirs.read().dir_of_hashed_books.join(book_hash).with_extension("png")
    }
  }
  pub fn get_book_data(&self, db: &DB) -> Result<BookData> {
    Ok(match &self.book_data_pk {
      BookDataPK::UniqueSize(book_size) => { db.get_primary::<DataOfUnhashedBook>(book_size.clone())?.unwrap().book_data }
      BookDataPK::RepeatingSize(book_hash) => { db.get_primary::<DataOfHashedBook>(book_hash.clone())?.unwrap().book_data }
    })
  }
  pub fn get_by_path(path_to_book: &BookPath, db: &DB) -> Result<Option<Book>> {
    Ok(db.get_primary::<Book>(path_to_book.clone())?)
  }
  pub(crate) fn get_by_size(book_size: BookSize, db: &DB) -> Result<Vec<Book>> {
    let mut res = vec![];
    match db.get_primary::<DataOfUnhashedBook>(book_size)? {
      None => {
        for i in DataOfHashedBook::find_by_size(book_size, db)? {
          for book_path in i.book_data.books_pk {
            let book = Book::get_by_path(&book_path, db)?.unwrap();
            res.push(book);
          }
        }
      }
      Some(data_of_book_with_such_size) => {
        let book = Book::get_by_path(&data_of_book_with_such_size.book_data.books_pk[0], db)?.unwrap();
        res.push(book);
      }
    }
    Ok(res)
  }
  pub(crate) fn get_books_located_in_dir(path_to_dir: String, db: &DB) -> Result<Vec<Book>> {
    Ok(db.scan_primary_by::<BookPath, Book>(path_to_dir)?)
  }
  pub(crate) fn update_books_directory(old_dir_path: &PathBuf, new_dir_path: &PathBuf, db: &DB) -> Result<()> {
    let start_time = std::time::Instant::now();
    for old_book in Self::get_books_located_in_dir(old_dir_path.to_str().unwrap().to_string(), db)? {
      let mut new_book = old_book.clone();
      new_book.dir_name = new_dir_path.file_name().unwrap().to_str().unwrap().to_string();
      new_book.path_to_dir = new_dir_path.to_str().unwrap().to_string();
      new_book.full_path = new_dir_path.join(&old_book.book_name).to_str().unwrap().to_string();
      db.update(old_book, new_book)?
    }
    let total_time = start_time.elapsed();
    info!("Function update_books_directory executed in: {:?}", &total_time);
    Ok(())
  }
  pub fn get_all_existing_books(db: &DB) -> Result<Vec<Book>> {
    let all_books: Vec<Book> = db.scan_primary()?;
    let res = all_books.into_iter().filter(|book| {
      let book_data = match &book.book_data_pk {
        BookDataPK::UniqueSize(book_size) => { db.get_primary::<DataOfUnhashedBook>(book_size.clone()).unwrap().unwrap().book_data }
        BookDataPK::RepeatingSize(book_hash) => { db.get_primary::<DataOfHashedBook>(book_hash.clone()).unwrap().unwrap().book_data }
      };
      book_data.is_deleted == false
    }).collect();
    Ok(res)
  }
}
impl Hash for Book {
  fn hash<H: Hasher>(&self, state: &mut H) { self.full_path.hash(state); }
}
impl PartialEq for Book {
  fn eq(&self, other: &Self) -> bool { self.full_path == other.full_path }
}
impl Eq for Book {}
