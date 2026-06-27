use crate::db::models::books::BookHash;
use crate::{
  db::models::{MutoolData, books::book::BookPath},
  types::HashSet,
};

use native_db::transaction::RwTransaction;
use native_db::*;
#[allow(unused_imports)]
use native_model::{Model, native_model};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
#[native_model(id = 3, version = 1)]
#[native_db]
pub struct BookHashes {
  #[primary_key]
  pub hash: BookHash,
  pub books: HashSet<BookPath>,
  pub mutool_data: MutoolData,
}

impl BookHashes {
  #[allow(dead_code)]
  pub(crate) fn new(book_hash: BookHash, books: HashSet<BookPath>) -> Self {
    Self { hash: book_hash, books, mutool_data: MutoolData::default() }
  }
  fn get_by_hash(
    book_hash: BookHash, rw_t: &RwTransaction<'_>,
  ) -> Result<Option<BookHashes>, anyhow::Error> {
    Ok(rw_t.get().primary::<BookHashes>(book_hash)?)
  }
  #[allow(dead_code)]
  pub(crate) fn insert_book(
    book_hash: BookHash, book_path: &BookPath, rw_t: &RwTransaction<'_>,
  ) -> anyhow::Result<()> {
    match Self::get_by_hash(book_hash.clone(), rw_t)? {
      Some(old_self) => {
        let mut updated_self = old_self.clone();
        match updated_self.books.swap_take(book_path) {
          Some(mut old_book_path) => {
            if old_book_path.deleted {
              old_book_path.deleted = false;
              updated_self.books.insert(old_book_path);
            };
          }
          None => {
            updated_self.books.insert(book_path.clone());
          }
        };
        rw_t.update(old_self, updated_self)?;
      }
      None => {
        let mut books = HashSet::default();
        books.insert(book_path.clone());
        rw_t.insert(Self::new(book_hash, books))?;
      }
    };
    Ok(())
  }
  pub(crate) fn remove_book(
    book_hash: BookHash, book_path: &BookPath, rw_t: &RwTransaction<'_>,
  ) -> anyhow::Result<()> {
    if let Some(old_self) = Self::get_by_hash(book_hash, rw_t)? {
      let mut updated_self = old_self.clone();
      if updated_self.books.swap_remove(book_path) {
        match updated_self.books.is_empty() {
          true => {
            rw_t.remove(old_self)?;
          }
          false => {
            rw_t.update(old_self, updated_self)?;
          }
        };
      }
    };
    Ok(())
  }
  #[allow(dead_code)]
  pub(crate) fn mark_book_as_deleted(
    book_hash: BookHash, book_path: &BookPath, rw_t: &RwTransaction<'_>,
  ) -> anyhow::Result<()> {
    if let Some(old_self) = Self::get_by_hash(book_hash, rw_t)? {
      let mut updated_self = old_self.clone();
      if let Some(mut inn_book_path) = updated_self.books.swap_take(book_path) {
        inn_book_path.mark_as_deleted();
        updated_self.books.insert(inn_book_path);
        rw_t.update(old_self, updated_self)?;
      };
    };
    Ok(())
  }
  pub(crate) fn update_book_path(
    book_hash: BookHash, old_book_path: &BookPath, new_book_path: &BookPath,
    rw_t: &RwTransaction<'_>,
  ) -> anyhow::Result<()> {
    if let Some(old_self) = Self::get_by_hash(book_hash, rw_t)? {
      let mut updated_self = old_self.clone();
      if updated_self.books.swap_remove(old_book_path) {
        updated_self.books.insert(new_book_path.clone());
        rw_t.update(old_self, updated_self)?;
      };
    };
    Ok(())
  }
}
