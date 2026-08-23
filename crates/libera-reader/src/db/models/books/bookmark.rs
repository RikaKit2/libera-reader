use crate::db::DB;
use crate::db::models::books::book::{Book, BookPath};
use anyhow::Result;
use gpui::SharedString;
use native_db::*;
use native_model::{Model, native_model};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct BookMark {
  pub title: SharedString,
  pub content: SharedString,
  pub page_number: u32,
  pub time_created: SharedString,
  pub time_updated: SharedString,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[native_model(id = 5, version = 1)]
#[native_db]
pub struct BookBookmarks {
  #[primary_key]
  pub book_path: BookPath,
  pub items: Vec<BookMark>,
}

impl BookBookmarks {
  pub fn new(book_path: BookPath, items: Vec<BookMark>) -> Self {
    Self { book_path, items }
  }

  /// Get all bookmarks for a book.
  pub fn get(db: &DB, book_path: BookPath) -> Result<Vec<BookMark>> {
    Ok(db.get_primary::<Self>(book_path)?.map(|b| b.items).unwrap_or_default())
  }

  /// Add a bookmark to a book.
  pub fn add(db: &DB, book_path: BookPath, bookmark: BookMark) -> Result<Option<Book>> {
    db.rw_t(|rw_t| {
      let Some(mut book) = rw_t.get().primary::<Book>(book_path.clone())? else {
        return Ok(None);
      };

      let old_bookmarks = rw_t.get().primary::<Self>(book_path.clone())?;
      let mut new_bookmarks =
        old_bookmarks.clone().unwrap_or_else(|| Self::new(book_path.clone(), Vec::new()));
      new_bookmarks.items.push(bookmark);

      let old_book = book.clone();
      book.bookmark_count = new_bookmarks.items.len();

      if let Some(old) = old_bookmarks {
        rw_t.update::<Self>(old, new_bookmarks)?;
      } else {
        rw_t.insert::<Self>(new_bookmarks)?;
      }
      rw_t.update::<Book>(old_book, book.clone())?;
      Ok(Some(book))
    })
  }

  /// Update an existing bookmark in a book.
  pub fn update(db: &DB, book_path: BookPath, bookmark: BookMark) -> Result<Option<Book>> {
    db.rw_t(|rw_t| {
      let Some(book) = rw_t.get().primary::<Book>(book_path.clone())? else {
        return Ok(None);
      };
      let Some(mut bookmarks) = rw_t.get().primary::<Self>(book_path)? else {
        return Ok(None);
      };
      let old_bookmarks = bookmarks.clone();
      let Some(existing) =
        bookmarks.items.iter_mut().find(|b| b.time_created == bookmark.time_created)
      else {
        return Ok(None);
      };
      *existing = bookmark;
      rw_t.update::<Self>(old_bookmarks, bookmarks)?;
      Ok(Some(book))
    })
  }

  /// Remove a bookmark from a book by creation timestamp.
  pub fn remove(db: &DB, book_path: BookPath, time_created: &str) -> Result<Option<Book>> {
    db.rw_t(|rw_t| {
      let Some(mut book) = rw_t.get().primary::<Book>(book_path.clone())? else {
        return Ok(None);
      };
      let Some(mut bookmarks) = rw_t.get().primary::<Self>(book_path)? else {
        return Ok(None);
      };
      let old_bookmarks = bookmarks.clone();
      let old_len = bookmarks.items.len();
      bookmarks.items.retain(|b| b.time_created.as_ref() != time_created);
      if bookmarks.items.len() == old_len {
        return Ok(None);
      }

      let old_book = book.clone();
      book.bookmark_count = bookmarks.items.len();

      if bookmarks.items.is_empty() {
        rw_t.remove::<Self>(old_bookmarks)?;
      } else {
        rw_t.update::<Self>(old_bookmarks, bookmarks)?;
      }
      rw_t.update::<Book>(old_book, book.clone())?;
      Ok(Some(book))
    })
  }
}
