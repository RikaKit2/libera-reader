use std::{
  hash::{Hash, Hasher},
  path::{Path, PathBuf},
};

use gpui::SharedString;
use serde::{Deserialize, Serialize};

use crate::db::models::{BookMark, UserData};
use native_db::*;
#[allow(unused_imports)]
use native_model::{Model, native_model};

pub(crate) type BookName = SharedString;
mod book_dir;
mod book_ext;
mod book_path;
mod book_size;
mod snapshot;

pub use book_dir::BookDir;
pub use book_ext::BookExt;
pub use book_path::BookPath;
pub use book_size::BookSize;
pub use snapshot::BookSnapshot;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[native_model(id = 1, version = 2)]
#[native_db]
pub struct Book {
  #[primary_key]
  pub id: String, // book_path.full_path_string()
  #[secondary_key]
  pub parent_dir: String, // book_path.parent_dir.full_path()

  pub book_path: BookPath,
  pub book_size: BookSize,
  pub user_data: UserData,
  #[serde(default)]
  pub bookmarks: Vec<BookMark>,
}

impl Book {
  pub(crate) fn new(book_path: BookPath) -> anyhow::Result<Self> {
    let book_size = book_path.get_book_size().unwrap();
    Ok(Self {
      id: book_path.full_path_string().to_string(),
      parent_dir: book_path.parent_dir.full_path().to_string(),
      book_path,
      book_size,
      user_data: UserData::default(),
      bookmarks: Vec::new(),
    })
  }

  pub fn exists_on_disk(&self) -> bool {
    self.book_path.exists_on_disk()
  }
  #[allow(dead_code)]
  pub(crate) fn pathbuf(&self) -> PathBuf {
    self.book_path.as_pathbuf()
  }
  #[allow(dead_code)]
  pub(crate) fn full_path_str(&self) -> SharedString {
    self.pathbuf().to_str().unwrap().to_string().into()
  }
  pub fn can_delete(&self) -> bool {
    !self.user_data.favorite && self.user_data.last_opened == 0 && self.bookmarks.is_empty()
  }

  /// Whether a usable thumbnail PNG currently exists on disk for this book.
  ///
  /// This is the **source of truth** for the UI's "does this book have a cover?"
  /// question. The previous `Book::has_thumbnail: bool` field was a cache that
  /// could desync from reality after manual cache wipes, external extraction,
  /// or failed extractions that left the flag set. We now consult the actual
  /// filesystem state through `BookSizes` / `BookHashes` metadata, which is
  /// exactly what the UI's `ThumbnailCache` does.
  ///
  /// See `documentation/ram.md` and the plan's step 5 for the rationale.
  pub fn has_thumbnail_on_disk(
    &self, db: &crate::db::DB, thumbnails_dir: &std::path::Path,
  ) -> bool {
    self.get_thumbnail_png_path(db, thumbnails_dir).ok().flatten().is_some()
  }

  pub fn add_bookmark(&mut self, bookmark: BookMark) {
    self.bookmarks.push(bookmark);
  }

  pub fn update_bookmark(&mut self, bookmark: BookMark) -> bool {
    if let Some(existing) =
      self.bookmarks.iter_mut().find(|b| b.time_created == bookmark.time_created)
    {
      *existing = bookmark;
      true
    } else {
      false
    }
  }

  pub fn remove_bookmark(&mut self, time_created: &str) -> bool {
    let old_len = self.bookmarks.len();
    self.bookmarks.retain(|b| b.time_created.as_ref() != time_created);
    old_len != self.bookmarks.len()
  }

  pub(crate) fn mark_as_deleted(&mut self) {
    self.book_path.mark_as_deleted();
  }

  /// Compute the path to the thumbnail PNG on disk by traversing BookSizes/BookHashes metadata.
  /// Returns None if no PNG file exists at the predicted path.
  pub fn get_thumbnail_png_path(
    &self, db: &crate::db::DB, thumbnails_dir: &Path,
  ) -> anyhow::Result<Option<PathBuf>> {
    let crate::db::models::books::book::BookSize::BYTES(size_bytes) = self.book_size;
    let unhashed_path = thumbnails_dir.join("unhashed_books").join(format!("{}.png", size_bytes));

    if let Some(book_sizes) =
      db.get_primary::<crate::db::models::books::book_sizes::BookSizes>(self.book_size)?
    {
      match &book_sizes.book_type {
        crate::db::models::books::BookType::UniqueSize { .. } => {
          if unhashed_path.exists() {
            return Ok(Some(unhashed_path));
          }
        }
        crate::db::models::books::BookType::DuplicateSize(map) => {
          if let Some(dup_data) = map.get(&self.book_path) {
            match dup_data {
              crate::db::models::books::DuplicateBookData::BookHash(hash) => {
                let hashed_path =
                  thumbnails_dir.join("hashed_books").join(format!("{}.png", hash.0));
                if hashed_path.exists() {
                  return Ok(Some(hashed_path));
                }
              }
              crate::db::models::books::DuplicateBookData::MutoolData(_) => {
                // No hash computed yet — might still have the unhashed PNG
                if unhashed_path.exists() {
                  return Ok(Some(unhashed_path));
                }
              }
            }
          } else {
            // Book path not found in map — unusual, try unhashed
            if unhashed_path.exists() {
              return Ok(Some(unhashed_path));
            }
          }
        }
      }
    } else {
      // Fallback: try unhashed path
      if unhashed_path.exists() {
        return Ok(Some(unhashed_path));
      }
    }

    Ok(None)
  }

  /// Retrieve mutool error from the database if extraction failed
  pub fn get_mutool_error(
    &self, db: &crate::db::DB,
  ) -> anyhow::Result<Option<mutool::mutool_error::MuToolError>> {
    db.rt(|r| {
      if let Some(book_sizes) =
        r.get().primary::<crate::db::models::books::book_sizes::BookSizes>(self.book_size)?
      {
        match &book_sizes.book_type {
          crate::db::models::books::BookType::UniqueSize { mutool_data, .. } => {
            return Ok(mutool_data.as_ref().and_then(|m| m.mutool_err.clone()));
          }
          crate::db::models::books::BookType::DuplicateSize(map) => {
            if let Some(dup_data) = map.get(&self.book_path) {
              match dup_data {
                crate::db::models::books::DuplicateBookData::MutoolData(m) => {
                  return Ok(m.as_ref().and_then(|m| m.mutool_err.clone()));
                }
                crate::db::models::books::DuplicateBookData::BookHash(hash) => {
                  if let Some(book_hashes) =
                    r.get()
                      .primary::<crate::db::models::books::book_hashes::BookHashes>(hash.clone())?
                  {
                    return Ok(book_hashes.mutool_data.mutool_err.clone());
                  }
                }
              }
            }
          }
        }
      }
      Ok(None)
    })
  }
}

impl Hash for Book {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.id.hash(state);
  }
}
