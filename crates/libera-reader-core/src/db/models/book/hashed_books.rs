use std::path::PathBuf;

use gxhash::HashMapExt;
use native_db::*;
#[allow(unused_imports)]
use native_model::{Model, native_model};
use serde::{Deserialize, Serialize};
use utils::calc_file_hash;

use crate::{
  db::{
    DB,
    models::{BookFsData, MutoolData, UniqueBook, UserData},
  },
  types::{BookHash, BookPath, BookSize, BookType, HashMap, NotCachedBooks},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[native_model(id = 3, version = 1)]
#[native_db]
pub struct HashedBooks {
  #[primary_key]
  pub book_hash: BookHash,
  #[secondary_key]
  pub size: BookSize,
  pub books: HashMap<BookPath, BookFsData>,
  pub mutool_data: MutoolData,
  pub user_data: UserData,
  pub deleted: bool,
}
impl HashedBooks {
  pub(crate) fn new(book_hash: BookHash, book_size: BookSize, books: HashMap<BookPath, BookFsData>) -> Self {
    let mutool_data = match book_size == 0 {
      true => MutoolData::default(),
      false => MutoolData::new_if_size_eq_zero(),
    };
    HashedBooks { book_hash, size: book_size, books, mutool_data, user_data: UserData::default(), deleted: false }
  }
  pub(crate) async fn insert_using_unique_book(unique_book: UniqueBook, db: &DB, not_cached_books: &NotCachedBooks) -> anyhow::Result<()> {
    let is_cached = unique_book.mutool_data.is_cached();
    let book_hash: BookHash = match unique_book.book_hash {
      Some(book_hash) => book_hash,
      None => calc_file_hash(unique_book.to_path_buf()).await.unwrap(),
    };
    let mut books = HashMap::new();
    books.insert(unique_book.full_path, unique_book.fs_data);
    let hashed_books = HashedBooks {
      book_hash: book_hash.clone(),
      size: unique_book.size,
      books,
      mutool_data: unique_book.mutool_data,
      user_data: unique_book.user_data,
      deleted: false,
    };
    db.insert::<Self>(hashed_books)?;
    if is_cached == false {
      not_cached_books.push(Box::new(BookType::Hashed(book_hash)));
    }
    Ok(())
  }
  pub(crate) async fn insert(pathbuf: &PathBuf, db: &DB, book_size: BookSize, not_cached_books: &NotCachedBooks) -> anyhow::Result<()> {
    let book_path_str = pathbuf.to_str().unwrap().to_string();
    let book_fs_data = BookFsData::from_pathbuf(pathbuf);
    let book_hash: BookHash = calc_file_hash(pathbuf).await?;
    match db.get_primary::<Self>(book_hash.clone())? {
      Some(old_data) => {
        let is_cached = old_data.mutool_data.is_cached();
        let mut modifyed_data = old_data.clone();
        modifyed_data.books.insert(book_path_str, book_fs_data);
        db.update(old_data, modifyed_data)?;
        if is_cached == false {
          not_cached_books.push(Box::new(BookType::Hashed(book_hash)));
        }
      }
      None => {
        let mut new_books_list = HashMap::new();
        new_books_list.insert(book_path_str, book_fs_data);
        db.insert::<Self>(Self::new(book_hash.clone(), book_size, new_books_list))?;
        not_cached_books.push(Box::new(BookType::Hashed(book_hash)));
      }
    };
    Ok(())
  }
  pub(crate) fn scan_by_size(book_size: BookSize, db: &DB) -> anyhow::Result<Vec<Self>> {
    db.scan_secondary_start_with::<BookSize, Self>(book_size, HashedBooksKey::size)
  }
  pub(crate) fn mark_as_deleted(&self, db: &DB) -> anyhow::Result<()> {
    let mut updated_data = self.clone();
    updated_data.deleted = true;
    db.update(self.clone(), updated_data)?;
    Ok(())
  }
  pub(crate) fn get_by_path(book_path: BookPath, db: &DB) -> anyhow::Result<Option<HashedBooks>> {
    let mut res: Option<HashedBooks> = None;
    let data = db.scan_primary::<Self>()?;
    for i in data {
      if i.books.contains_key(&book_path) {
        res = Some(i);
        break;
      }
    }
    Ok(res)
  }
  pub(crate) fn remove(&self, db: &DB) -> anyhow::Result<()> {
    match self.user_data.can_delete() {
      true => db.remove::<Self>(self.clone())?,
      false => self.mark_as_deleted(db)?,
    };
    Ok(())
  }
  pub(crate) fn remove_by_path(mut self, book_path: &BookPath, db: &DB) -> anyhow::Result<bool> {
    let is_removed: bool;
    match self.books.get(book_path) {
      Some(_) => {
        let book_paths_count = self.books.len();
        if book_paths_count == 1 {
          self.remove(db)?;
        } else if book_paths_count > 1 {
          self.books.remove(book_path);
        };
        is_removed = true;
      }
      None => {
        is_removed = false;
      }
    };

    Ok(is_removed)
  }
}
