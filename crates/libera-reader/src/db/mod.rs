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

  #[test]
  fn test_bookmarks_lifecycle_on_path_update_and_remove() -> Result<()> {
    let tmp_dir = tempfile::tempdir()?;
    let db_path = tmp_dir.path().join("test_bm.redb");
    let db = DB::new(db_path)?;
    let app_dirs = crate::app_dirs::AppDirs::new(tmp_dir.path().to_path_buf()).unwrap();

    let old_file = tmp_dir.path().join("book_old.pdf");
    std::fs::write(&old_file, b"%PDF-1.4 dummy")?;
    let new_file = tmp_dir.path().join("book_new.pdf");

    let old_book_path = BookPath::new(&old_file).unwrap();
    let new_book_path = BookPath::new(&new_file).unwrap();

    // 1. Insert book into DB
    let book = Book::new(old_book_path.clone()).unwrap();
    db.rw_t(|rw_t| {
      crate::db::models::books::book_sizes::BookSizes::insert_book(&book, rw_t)?;
      rw_t.insert::<Book>(book.clone())?;
      Ok(())
    })?;

    // 2. Add a bookmark to the old path
    let bookmark = BookMark {
      title: "Page 1 Note".into(),
      content: "Important".into(),
      page_number: 1,
      time_created: "2026-10-21T10:00:00Z".into(),
      time_updated: "2026-10-21T10:00:00Z".into(),
    };
    BookBookmarks::add(&db, old_book_path.clone(), bookmark.clone())?;
    assert_eq!(BookBookmarks::get(&db, old_book_path.clone())?.len(), 1);

    // 3. Move book (rename) -> update_book_path
    db.rw_t(|rw_t| {
      crate::services::notify_service::fs_handlers::update_book_path(
        old_file.clone(),
        new_file.clone(),
        rw_t,
      )
    })?;

    // Check bookmarks migrated
    let old_bms = BookBookmarks::get(&db, old_book_path.clone())?;
    assert!(old_bms.is_empty(), "Old path should have no bookmarks after move");
    let new_bms = BookBookmarks::get(&db, new_book_path.clone())?;
    assert_eq!(new_bms.len(), 1, "New path should retain the bookmark");
    assert_eq!(new_bms[0].title, "Page 1 Note");

    // 4. File removed while having bookmarks -> soft delete (preserves user data)
    let status = db.rw_t(|rw_t| {
      crate::services::notify_service::fs_handlers::remove_book_by_path(&new_file, rw_t, &app_dirs)
    })?;
    assert_eq!(status, crate::services::notify_service::RemoveStatus::MarkedAsDeleted);
    let soft_deleted_bms = BookBookmarks::get(&db, new_book_path.clone())?;
    assert_eq!(soft_deleted_bms.len(), 1, "Bookmarks preserved during soft delete");

    // 5. Remove bookmark so book becomes eligible for full delete (can_delete == true)
    BookBookmarks::remove(&db, new_book_path.clone(), &bookmark.time_created)?;
    // Insert an orphaned BookBookmarks record to ensure full delete purges it
    db.rw_t(|rw_t| {
      rw_t.insert::<BookBookmarks>(BookBookmarks::new(new_book_path.clone(), vec![]))?;
      Ok(())
    })?;
    assert!(db.get_primary::<BookBookmarks>(new_book_path.clone())?.is_some());

    // 6. Full delete -> removes Book and BookBookmarks from DB
    let status = db.rw_t(|rw_t| {
      crate::services::notify_service::fs_handlers::remove_book_by_path(&new_file, rw_t, &app_dirs)
    })?;
    assert_eq!(status, crate::services::notify_service::RemoveStatus::FullyDeleted);
    let fully_deleted_bms = db.get_primary::<BookBookmarks>(new_book_path.clone())?;
    assert!(fully_deleted_bms.is_none(), "Orphaned bookmarks purged on full delete");

    Ok(())
  }
}
