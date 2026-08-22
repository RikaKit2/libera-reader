pub mod models;

use crate::db::models::BookMark;
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
  models.define::<Book>()?;
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
    let mut builder = Builder::new();
    builder.set_cache_size(16 * 1024 * 1024); // Limit cache size to 16 MB to prevent memory bloat
    let db = if path_to_db.exists() {
      builder.open(&MODELS, &path_to_db)?
    } else {
      builder.create(&MODELS, &path_to_db)?
    };
    Ok(Self { db: Arc::new(RwLock::new(db)) })
  }

  pub fn compact(&self) -> Result<()> {
    self.db.write().unwrap().compact()?;
    Ok(())
  }

  pub fn get_book(&self, book_path: BookPath) -> Result<Option<Book>> {
    self.get_primary::<Book>(book_path.full_path_string().to_string())
  }

  pub fn update_book(&self, updated_book: Book) -> Result<()> {
    self.rw_t(|rw_t| {
      let old_book = rw_t
        .get()
        .primary::<Book>(updated_book.id.clone())?
        .ok_or_else(|| anyhow::anyhow!("Book not found: {}", updated_book.id))?;
      rw_t.update::<Book>(old_book, updated_book)?;
      Ok(())
    })
  }

  /// Scan ALL books in the database (flat)
  pub fn scan_all_books(&self) -> Result<Vec<Book>> {
    self.rt(scan_primary::<Book>)
  }

  /// Stream every book in the database to a callback without materializing
  /// the full `Vec<Book>` in memory at once. Used by the UI's initial load so
  /// a library of 10 000 books doesn't allocate 10 000 heavy `Book` structs
  /// simultaneously before they can be converted to `LightBook`s.
  ///
  /// The callback may return `Err` to abort iteration early.
  pub fn for_each_book<F>(&self, mut f: F) -> Result<()>
  where
    F: FnMut(Book) -> Result<()>,
  {
    self.rt(|r_txn| {
      let scan = r_txn.scan().primary()?;
      let iter = scan.all()?;
      for item in iter {
        let book: Book = item?;
        f(book)?;
      }
      Ok(())
    })
  }

  /// Scan books by parent_dir (filter in-memory)
  pub fn scan_books_by_parent_dir(&self, parent_dir: &str) -> Result<Vec<Book>> {
    let all = self.scan_all_books()?;
    Ok(all.into_iter().filter(|b| b.parent_dir == parent_dir).collect())
  }

  pub fn add_bookmark(&self, book_path: BookPath, bookmark: BookMark) -> Result<Option<Book>> {
    self.rw_t(|rw_t| {
      let Some(mut book) = rw_t.get().primary::<Book>(book_path.full_path_string().to_string())?
      else {
        return Ok(None);
      };
      let old_book = book.clone();
      book.add_bookmark(bookmark);
      rw_t.update::<Book>(old_book, book.clone())?;
      Ok(Some(book))
    })
  }

  pub fn update_bookmark(&self, book_path: BookPath, bookmark: BookMark) -> Result<Option<Book>> {
    self.rw_t(|rw_t| {
      let Some(mut book) = rw_t.get().primary::<Book>(book_path.full_path_string().to_string())?
      else {
        return Ok(None);
      };
      let old_book = book.clone();
      if !book.update_bookmark(bookmark) {
        return Ok(None);
      }
      rw_t.update::<Book>(old_book, book.clone())?;
      Ok(Some(book))
    })
  }

  pub fn remove_bookmark(&self, book_path: BookPath, time_created: &str) -> Result<Option<Book>> {
    self.rw_t(|rw_t| {
      let Some(mut book) = rw_t.get().primary::<Book>(book_path.full_path_string().to_string())?
      else {
        return Ok(None);
      };
      let old_book = book.clone();
      if !book.remove_bookmark(time_created) {
        return Ok(None);
      }
      rw_t.update::<Book>(old_book, book.clone())?;
      Ok(Some(book))
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

  #[allow(dead_code)]
  pub(crate) fn remove<T: ToInput>(&self, item: T) -> Result<(), Box<db_type::Error>> {
    let lock = self.db.write().unwrap();
    let rw_conn = lock.rw_transaction().map_err(Box::new)?;
    rw_conn.remove(item).map_err(Box::new)?;
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
    let lock = self.db.read().unwrap();
    let r_txn = lock.r_transaction()?;
    let result = func(&r_txn)?;
    Ok(result)
  }
}

pub(crate) fn scan_primary<T: ToInput>(r: &RTransaction<'_>) -> Result<Vec<T>> {
  Ok(r.scan().primary()?.all()?.try_collect()?)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::db::models::UserData;
  use crate::db::models::books::book::{BookDir, BookExt, BookSize};

  #[test]
  fn test_db_crud_operations() -> Result<()> {
    let tmp_dir = tempfile::tempdir()?;
    let db_path = tmp_dir.path().join("test.redb");
    let db = DB::new(db_path)?;

    let book_dir = BookDir::new(tmp_dir.path().to_path_buf());
    let book_path = BookPath {
      parent_dir: book_dir.clone(),
      name: "sample".into(),
      ext: BookExt::PDF("pdf".into()),
      deleted: false,
    };

    let book = Book {
      id: book_path.full_path_string().to_string(),
      parent_dir: book_dir.full_path().to_string(),
      book_path: book_path.clone(),
      book_size: BookSize::BYTES(1024),
      user_data: UserData::default(),
      bookmarks: Vec::new(),
    };

    db.insert(book.clone())?;

    let retrieved = db.get_book(book_path.clone())?;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, book.id);

    let bookmark = BookMark {
      title: "Chapter 1".into(),
      content: "First note".into(),
      page_number: 5,
      time_created: "2026-08-22T00:00:00Z".into(),
      time_updated: "2026-08-22T00:00:00Z".into(),
    };
    db.add_bookmark(book_path.clone(), bookmark.clone())?;

    let with_bookmark = db.get_book(book_path.clone())?.unwrap();
    assert_eq!(with_bookmark.bookmarks.len(), 1);
    assert_eq!(with_bookmark.bookmarks[0].title, "Chapter 1");

    let books_in_dir = db.scan_books_by_parent_dir(book_dir.full_path().as_ref())?;
    assert_eq!(books_in_dir.len(), 1);

    db.remove_bookmark(book_path.clone(), &bookmark.time_created)?;
    let without_bookmark = db.get_book(book_path.clone())?.unwrap();
    assert_eq!(without_bookmark.bookmarks.len(), 0);

    db.compact()?;
    Ok(())
  }
}
