pub mod models;

use crate::db::models::BookBookmarks;
use crate::db::models::books::book::Book;

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
  models.define::<BookBookmarks>()?;
  Ok(models)
}

static MODELS: Lazy<Models> = Lazy::new(|| get_models().unwrap());

#[derive(Clone)]
pub struct DB {
  db: Arc<RwLock<Database<'static>>>,
}

impl gpui::Global for DB {}
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
  use crate::db::models::books::book::{BookDir, BookExt, BookPath, BookSize};
  use crate::db::models::{BookMark, UserData};
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
      parent_dir: book_dir.clone(),
      book_path: book_path.clone(),
      book_size: BookSize::BYTES(1024),
      user_data: UserData::default(),
      bookmark_count: 0,
    };

    db.insert(book.clone())?;

    let retrieved = Book::get(&db, book_path.clone())?;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().book_path, book.book_path);

    let bookmark = BookMark {
      title: "Chapter 1".into(),
      content: "First note".into(),
      page_number: 5,
      time_created: "2026-08-22T00:00:00Z".into(),
      time_updated: "2026-08-22T00:00:00Z".into(),
    };
    BookBookmarks::add(&db, book_path.clone(), bookmark.clone())?;

    let with_bookmark = Book::get(&db, book_path.clone())?.unwrap();
    assert_eq!(with_bookmark.bookmark_count, 1);

    let bookmarks = BookBookmarks::get(&db, book_path.clone())?;
    assert_eq!(bookmarks.len(), 1);
    assert_eq!(bookmarks[0].title, "Chapter 1");

    let books_in_dir = Book::scan_by_parent_dir(&db, book_dir.full_path().as_ref())?;
    assert_eq!(books_in_dir.len(), 1);

    BookBookmarks::remove(&db, book_path.clone(), &bookmark.time_created)?;
    let without_bookmark = Book::get(&db, book_path.clone())?.unwrap();
    assert_eq!(without_bookmark.bookmark_count, 0);
    let bookmarks_after = BookBookmarks::get(&db, book_path.clone())?;
    assert_eq!(bookmarks_after.len(), 0);
    Ok(())
  }
}
