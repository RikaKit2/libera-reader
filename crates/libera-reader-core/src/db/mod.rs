pub mod models;

use crate::db::models::BookMark;
use crate::db::models::books::Books;
use crate::db::models::books::book::{Book, BookPath};

use crate::db::models::books::book_hashes::BookHashes;
use crate::db::models::books::book_sizes::BookSizes;
use crate::db::models::settings::Settings;
use anyhow::Result;
use itertools::Itertools;
use native_db::transaction::{RTransaction, RwTransaction};
use native_db::{Builder, Database, Models, ToInput, ToKey, db_type};
use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

fn get_models() -> Result<Models> {
  let mut models = Models::new();
  models.define::<Books>()?;
  models.define::<Settings>()?;
  models.define::<BookSizes>()?;
  models.define::<BookHashes>()?;
  Ok(models)
}

static MODELS: Lazy<Models> = Lazy::new(|| get_models().unwrap());

#[derive(Clone)]
pub struct DB {
  db: Arc<RwLock<Database<'static>>>,
}

impl DB {
  pub fn new(path_to_db: PathBuf) -> Result<Self> {
    let db = if path_to_db.exists() {
      Builder::new().open(&MODELS, &path_to_db)?
    } else {
      Builder::new().create(&MODELS, &path_to_db)?
    };
    Ok(Self { db: Arc::new(RwLock::new(db)) })
  }

  pub fn compact(&self) -> Result<()> {
    self.db.write().unwrap().compact()?;
    Ok(())
  }

  pub fn get_book(&self, book_path: BookPath) -> Result<Option<Book>> {
    self.rt(|r| Books::get_by_path(book_path, r))
  }

  pub fn update_book(&self, updated_book: Book) -> Result<()> {
    self.rw_t(|rw_t| {
      let parent_dir = updated_book.book_path.parent_dir.clone();
      let key = updated_book.book_path.file_name();

      match Books::get_by_parent_dir_rw(parent_dir, rw_t)? {
        Some(old_books) => {
          let mut new_books = old_books.clone();
          new_books.storage.insert(key, updated_book);
          rw_t.update(old_books, new_books)?;
        }
        None => {
          Books::insert_book(updated_book, rw_t)?;
        }
      }

      Ok(())
    })
  }

  pub fn add_bookmark(&self, book_path: BookPath, bookmark: BookMark) -> Result<Option<Book>> {
    self.rw_t(|rw_t| {
      let parent_dir = book_path.parent_dir.clone();
      let key = book_path.file_name();

      let Some(old_books) = Books::get_by_parent_dir_rw(parent_dir, rw_t)? else {
        return Ok(None);
      };
      let mut new_books = old_books.clone();

      let Some(book) = new_books.storage.get_mut(&key) else {
        return Ok(None);
      };

      book.add_bookmark(bookmark);
      let updated_book = book.clone();
      rw_t.update(old_books, new_books)?;
      Ok(Some(updated_book))
    })
  }

  pub fn update_bookmark(&self, book_path: BookPath, bookmark: BookMark) -> Result<Option<Book>> {
    self.rw_t(|rw_t| {
      let parent_dir = book_path.parent_dir.clone();
      let key = book_path.file_name();

      let Some(old_books) = Books::get_by_parent_dir_rw(parent_dir, rw_t)? else {
        return Ok(None);
      };
      let mut new_books = old_books.clone();

      let Some(book) = new_books.storage.get_mut(&key) else {
        return Ok(None);
      };

      if !book.update_bookmark(bookmark) {
        return Ok(None);
      }

      let updated_book = book.clone();
      rw_t.update(old_books, new_books)?;
      Ok(Some(updated_book))
    })
  }

  pub fn remove_bookmark(&self, book_path: BookPath, time_created: &str) -> Result<Option<Book>> {
    self.rw_t(|rw_t| {
      let parent_dir = book_path.parent_dir.clone();
      let key = book_path.file_name();

      let Some(old_books) = Books::get_by_parent_dir_rw(parent_dir, rw_t)? else {
        return Ok(None);
      };
      let mut new_books = old_books.clone();

      let Some(book) = new_books.storage.get_mut(&key) else {
        return Ok(None);
      };

      if !book.remove_bookmark(time_created) {
        return Ok(None);
      }

      let updated_book = book.clone();
      rw_t.update(old_books, new_books)?;
      Ok(Some(updated_book))
    })
  }

  pub(crate) fn get_primary<T: ToInput>(&self, key: impl ToKey) -> Result<Option<T>> {
    Ok(self.db.read().unwrap().r_transaction()?.get().primary(key)?)
  }

  pub(crate) fn insert<T: ToInput>(&self, item: T) -> Result<(), Box<db_type::Error>> {
    let lock = self.db.write().unwrap();
    let rw_conn = lock.rw_transaction().map_err(Box::new)?;
    rw_conn.insert(item).map_err(Box::new)?;
    rw_conn.commit().map_err(Box::new)
  }

  pub(crate) fn update<T: ToInput>(
    &self, old_data: T, new_data: T,
  ) -> Result<(), Box<db_type::Error>> {
    let lock = self.db.write().unwrap();
    let rw_conn = lock.rw_transaction().map_err(Box::new)?;
    rw_conn.update(old_data, new_data).map_err(Box::new)?;
    rw_conn.commit().map_err(Box::new)
  }

  pub fn rw_t<F, T>(&self, func: F) -> Result<T>
  where
    F: FnOnce(&RwTransaction) -> Result<T>,
  {
    let lock = self.db.write().unwrap();
    let rw_txn = lock.rw_transaction()?;
    let result = func(&rw_txn)?;
    rw_txn.commit()?;
    Ok(result)
  }

  pub fn rt<F, T>(&self, func: F) -> Result<T>
  where
    F: FnOnce(&RTransaction) -> Result<T>,
  {
    let lock = self.db.write().unwrap();
    let r_txn = lock.r_transaction()?;
    let result = func(&r_txn)?;
    Ok(result)
  }
}

pub(crate) fn scan_primary<T: ToInput>(r: &RTransaction<'_>) -> Result<Vec<T>> {
  Ok(r.scan().primary()?.all()?.try_collect()?)
}
