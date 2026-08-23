use std::{
  hash::{Hash, Hasher},
  path::{Path, PathBuf},
};

use gpui::SharedString;
use mutool::mutool_error::MuToolError;
use serde::{Deserialize, Serialize};

use crate::db::models::{
  UserData,
  books::{BookType, DuplicateBookData, book_hashes::BookHashes, book_sizes::BookSizes},
};
use native_db::*;
#[allow(unused_imports)]
use native_model::{Model, native_model};

pub(crate) type BookName = SharedString;
mod book_dir;
mod book_ext;
mod book_path;
mod book_size;

pub use book_dir::BookDir;
pub use book_ext::BookExt;
pub use book_path::BookPath;
pub use book_size::BookSize;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[native_model(id = 1, version = 1)]
#[native_db]
pub struct Book {
  #[primary_key]
  pub book_path: BookPath,
  #[secondary_key]
  pub parent_dir: BookDir,

  pub book_size: BookSize,
  pub user_data: UserData,
  #[serde(default)]
  pub bookmark_count: usize,
}

impl Book {
  pub(crate) fn new(book_path: BookPath) -> anyhow::Result<Self> {
    let book_size = book_path.get_book_size().unwrap();
    let parent_dir = book_path.parent_dir.clone();
    Ok(Self { book_path, parent_dir, book_size, user_data: UserData::default(), bookmark_count: 0 })
  }

  #[inline]
  pub fn full_path_string(&self) -> SharedString {
    self.book_path.full_path_string()
  }

  #[inline]
  pub fn parent_dir(&self) -> &BookDir {
    &self.parent_dir
  }

  #[inline]
  pub fn parent_dir_str(&self) -> SharedString {
    self.parent_dir.full_path()
  }

  /// Fetch a book by its BookPath from the database
  pub fn get(db: &crate::db::DB, book_path: BookPath) -> anyhow::Result<Option<Self>> {
    db.get_primary::<Self>(book_path)
  }

  /// Update an existing book in the database
  pub fn update(db: &crate::db::DB, updated_book: Self) -> anyhow::Result<()> {
    db.rw_t(|rw_t| {
      let old_book = rw_t
        .get()
        .primary::<Self>(updated_book.book_path.clone())?
        .ok_or_else(|| anyhow::anyhow!("Book not found: {:?}", updated_book.full_path_string()))?;
      rw_t.update::<Self>(old_book, updated_book)?;
      Ok(())
    })
  }

  /// Scan ALL books in the database (flat)
  pub fn scan_all(db: &crate::db::DB) -> anyhow::Result<Vec<Self>> {
    db.rt(crate::db::scan_primary::<Self>)
  }

  /// Stream every book in the database to a callback without materializing
  /// the full `Vec<Book>` in memory at once.
  pub fn for_each<F>(db: &crate::db::DB, mut f: F) -> anyhow::Result<()>
  where
    F: FnMut(Self) -> anyhow::Result<()>,
  {
    db.rt(|r_txn| {
      let scan = r_txn.scan().primary()?;
      for item in scan.all()? {
        f(item?)?;
      }
      Ok(())
    })
  }

  /// Scan books filtered by parent directory
  pub fn scan_by_parent_dir(db: &crate::db::DB, parent_dir: &str) -> anyhow::Result<Vec<Self>> {
    let all = Self::scan_all(db)?;
    Ok(all.into_iter().filter(|b| b.parent_dir_str().as_ref() == parent_dir).collect())
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
    !self.user_data.favorite && self.user_data.last_opened == 0 && self.bookmark_count == 0
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

  pub fn name(&self) -> &SharedString {
    &self.book_path.name
  }

  pub fn size_bytes(&self) -> u64 {
    let BookSize::BYTES(size) = self.book_size;
    size
  }

  pub fn format_title_pixel_perfect(title: &str) -> SharedString {
    let mut breakable_title = String::with_capacity(title.len() * 4);
    for ch in title.chars() {
      breakable_title.push(ch);
      breakable_title.push('\u{200B}');
    }
    breakable_title.into()
  }

  pub fn formatted_title(&self) -> SharedString {
    Self::format_title_pixel_perfect(self.book_path.display_name().as_ref())
  }

  pub fn size_label(&self) -> SharedString {
    format!("File size: {} MB", self.size_bytes() / 1_048_576).into()
  }

  pub fn format_label(&self) -> SharedString {
    format!(
      "{}: {}",
      rust_i18n::t!("components.card.format_label"),
      self.book_path.ext.to_string().to_uppercase()
    )
    .into()
  }

  pub fn cover_btn_id(&self) -> SharedString {
    format!("cover_{}", self.full_path_string()).into()
  }

  pub fn fav_btn_id(&self) -> SharedString {
    format!("fav_{}", self.full_path_string()).into()
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

    if let Some(book_sizes) = db.get_primary::<BookSizes>(self.book_size)? {
      match &book_sizes.book_type {
        BookType::UniqueSize { .. } => {
          if unhashed_path.exists() {
            return Ok(Some(unhashed_path));
          }
        }
        BookType::DuplicateSize(map) => {
          if let Some(dup_data) = map.get(&self.book_path) {
            match dup_data {
              DuplicateBookData::BookHash(hash) => {
                let hashed_path =
                  thumbnails_dir.join("hashed_books").join(format!("{}.png", hash.0));
                if hashed_path.exists() {
                  return Ok(Some(hashed_path));
                }
              }
              DuplicateBookData::MutoolData(_) => {
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
  pub fn get_mutool_error(&self, db: &crate::db::DB) -> anyhow::Result<Option<MuToolError>> {
    db.rt(|r| {
      if let Some(book_sizes) = r.get().primary::<BookSizes>(self.book_size)? {
        match &book_sizes.book_type {
          BookType::UniqueSize { mutool_data, .. } => {
            return Ok(mutool_data.as_ref().and_then(|m| m.mutool_err.clone()));
          }
          BookType::DuplicateSize(map) => {
            if let Some(dup_data) = map.get(&self.book_path) {
              match dup_data {
                DuplicateBookData::MutoolData(m) => {
                  return Ok(m.as_ref().and_then(|m| m.mutool_err.clone()));
                }
                DuplicateBookData::BookHash(hash) => {
                  if let Some(book_hashes) = r.get().primary::<BookHashes>(hash.clone())? {
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
    self.book_path.hash(state);
  }
}
