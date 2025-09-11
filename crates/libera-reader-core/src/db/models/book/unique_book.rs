use std::{panic, path::PathBuf};

use native_db::*;
#[allow(unused_imports)]
use native_model::{Model, native_model};
use serde::{Deserialize, Serialize};

use crate::{
  db::{
    DB,
    models::{BookFsData, MutoolData, UserData},
  },
  types::{BookHash, BookPath, BookSize, BookType, NotCachedBooks},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[native_model(id = 2, version = 1)]
#[native_db]
pub struct UniqueBook {
  #[primary_key]
  pub size: BookSize,
  #[secondary_key(unique)]
  pub full_path: BookPath,
  pub book_hash: Option<BookHash>,
  pub fs_data: BookFsData,
  pub mutool_data: MutoolData,
  pub user_data: UserData,
  pub deleted: bool,
}
impl UniqueBook {
  pub(crate) fn new(book_size: BookSize, full_path: BookPath, book_fs_data: BookFsData) -> Self {
    let mutool_data = match book_size == 0 {
      true => MutoolData::default(),
      false => MutoolData::new_if_size_eq_zero(),
    };
    Self { size: book_size, full_path, book_hash: None, fs_data: book_fs_data, mutool_data, user_data: UserData::default(), deleted: false }
  }
  pub(crate) fn get_all(db: &DB) -> anyhow::Result<Vec<Self>> {
    Ok(db.scan_primary::<Self>()?)
  }
  pub(crate) async fn insert(pathbuf: &PathBuf, book_size: BookSize, db: &DB, not_cached_books: &NotCachedBooks) -> anyhow::Result<()> {
    let full_path = pathbuf.to_string_lossy().to_string();
    let book_fs_data = BookFsData::from_pathbuf(pathbuf);
    db.insert::<Self>(Self::new(book_size.clone(), full_path, book_fs_data))?;
    not_cached_books.push(Box::new(BookType::Unique(book_size)))?;
    Ok(())
  }
  pub(crate) fn get_by_size(book_size: BookSize, db: &DB) -> anyhow::Result<Option<Self>> {
    db.get_primary::<Self>(book_size)
  }
  pub(crate) fn get_by_path(book_path: BookPath, db: &DB) -> anyhow::Result<Option<Self>> {
    db.get_secondary::<Self>(book_path, UniqueBookKey::full_path)
  }
  pub(crate) fn to_path_buf(&self) -> PathBuf {
    PathBuf::from(&self.full_path)
  }
  pub(crate) fn mark_as_deleted(&self, db: &DB) -> anyhow::Result<()> {
    let mut updated_data = self.clone();
    updated_data.deleted = true;
    db.update(self.clone(), updated_data)?;
    Ok(())
  }
  pub(crate) fn remove(&self, db: &DB) -> anyhow::Result<()> {
    match self.user_data.can_delete() {
      true => db.remove::<UniqueBook>(self.clone())?,
      false => self.mark_as_deleted(db)?,
    };
    Ok(())
  }
  pub(crate) fn get_books_located_in_dir(path_to_dir: String, db: &DB) -> anyhow::Result<Vec<UniqueBook>> {
    db.scan_secondary_start_with::<BookPath, UniqueBook>(path_to_dir, UniqueBookKey::full_path)
  }
}
