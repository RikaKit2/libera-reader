mod insert_to_db;
mod delete;

use crate::db::crud;
use crate::db::crud::get_primary;
use crate::db::models::book_data::BookData;
use crate::db::models::book_data_pk::BookDataPK;
use crate::db::models::{DataOfHashedBook, DataOfUnhashedBook};
use crate::types::{BookPath, APP_DIRS, DB};
use anyhow::Result;
use itertools::Itertools;
use mutool_bindings::MUToolResult;
use native_db::*;
#[allow(unused_imports)]
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

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
  pub path_is_valid: bool,
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
      path_is_valid: true,
    }
  }
  pub(crate) fn mark_as_cached(self, db: &DB) -> Result<()> {
    self.book_data_pk.update(|book_data: &mut BookData| {
      book_data.thumbnail = Some(vec![]);
    }, db)
  }
  pub(crate) fn mark_as_broken(self, mutool_err: MUToolResult, db: &DB) -> Result<()> {
    self.book_data_pk.update(|book_data: &mut BookData| {
      book_data.mutool_err = Some(mutool_err);
    }, db)
  }
  pub(crate) fn path_to_thumbnail(&self, app_dirs: &APP_DIRS) -> PathBuf {
    match &self.book_data_pk {
      BookDataPK::UniqueSize(book_size) => app_dirs.read().unwrap().inn.dir_of_unhashed_books.join(book_size.to_string()).with_extension("png"),
      BookDataPK::RepeatingSize(book_hash) => app_dirs.read().unwrap().inn.dir_of_hashed_books.join(book_hash).with_extension("png")
    }
  }
  pub fn get_by_path(path_to_book: &BookPath, db: &DB) -> Result<Option<Book>> {
    Ok(get_primary::<Book>(path_to_book.clone(), db)?)
  }
  pub(crate) fn get_books_located_in_dir(path_to_dir: String, db: &DB) -> Result<Vec<Book>> {
    let r_conn = db.r_transaction()?;
    let books: Vec<Book> = r_conn.scan().primary().unwrap().start_with(path_to_dir)?.try_collect()?;
    Ok(books)
  }
  pub(crate) fn update_books_directory(old_dir_path: &PathBuf, new_dir_path: &PathBuf, db: &DB) -> Result<()> {
    for old_book in Self::get_books_located_in_dir(old_dir_path.to_str().unwrap().to_string(), db)? {
      let mut new_book = old_book.clone();
      new_book.dir_name = new_dir_path.file_name().unwrap().to_str().unwrap().to_string();
      new_book.path_to_dir = new_dir_path.to_str().unwrap().to_string();
      new_book.full_path = new_dir_path.join(&old_book.book_name).to_str().unwrap().to_string();
      crud::update(old_book, new_book, db)?
    }
    Ok(())
  }
  pub fn get_all_existing_books(db: &DB) -> Result<Vec<Book>> {
    let all_books: Vec<Book> = db.r_transaction()?.scan().primary()?.all()?.try_collect()?;
    let res = all_books.into_iter().filter(|book| {
      let book_data = match &book.book_data_pk {
        BookDataPK::UniqueSize(book_size) => { get_primary::<DataOfUnhashedBook>(book_size.clone(), db).unwrap().unwrap().book_data }
        BookDataPK::RepeatingSize(book_hash) => { get_primary::<DataOfHashedBook>(book_hash.clone(), db).unwrap().unwrap().book_data }
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
