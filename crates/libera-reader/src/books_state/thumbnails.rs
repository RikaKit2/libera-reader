use gpui::SharedString;
use std::path::{Path, PathBuf};

use crate::db::DB;
use crate::db::models::books::book::{Book, BookPath};
use crate::types::HashMap;

/// Resolved filesystem path for a thumbnail image (None if not present on disk).
pub type ThumbnailPath = Option<PathBuf>;

/// Map of book path (SharedString) to its resolved thumbnail path on disk.
pub type ThumbnailMap = HashMap<SharedString, ThumbnailPath>;

/// Resolution cache for book -> thumbnail PNG path on disk.
#[derive(Clone)]
pub struct ThumbnailCache {
  thumbnails_dir: PathBuf,
  entries: ThumbnailMap,
}

impl ThumbnailCache {
  pub fn new(thumbnails_dir: PathBuf) -> Self {
    Self { thumbnails_dir, entries: ThumbnailMap::default() }
  }

  pub fn thumbnails_dir(&self) -> &Path {
    &self.thumbnails_dir
  }

  /// Resolve the PNG path for one book from the database.
  /// Consults `BookSizes` / `BookHashes` and verifies presence on disk.
  pub fn resolve_for(db: &DB, thumbnails_dir: &Path, book_path: &BookPath) -> ThumbnailPath {
    let book = Book::get(db, book_path.clone()).ok()??;
    book.get_thumbnail_png_path(db, thumbnails_dir).ok().flatten().filter(|p| p.exists())
  }

  /// Resolve and insert/update a single entry by path. Returns the resolved path.
  pub fn upsert(&mut self, db: &DB, path_key: &SharedString) -> ThumbnailPath {
    let book_path = BookPath::from_full_path(path_key.as_ref());
    let path = Self::resolve_for(db, &self.thumbnails_dir, &book_path);
    self.entries.insert(path_key.clone(), path.clone());
    path
  }

  /// Insert a pre-resolved path (used on initial DB load).
  pub fn insert_resolved(&mut self, path_key: SharedString, path: ThumbnailPath) {
    self.entries.insert(path_key, path);
  }

  /// Remove a cached entry (on book removal or thumbnail extraction).
  pub fn remove(&mut self, path_key: &SharedString) {
    self.entries.swap_remove(path_key);
  }

  /// Rename an entry's key (on book rename/move).
  pub fn rename(&mut self, old_path: &SharedString, new_path: SharedString) {
    if let Some(path) = self.entries.swap_remove(old_path) {
      self.entries.insert(new_path, path);
    }
  }

  /// Look up the PNG path for a book path, resolving from the DB on cache miss.
  pub fn get_or_resolve(&mut self, db: &DB, path_key: &SharedString) -> ThumbnailPath {
    match self.entries.get(path_key) {
      Some(path) => path.clone(),
      None => self.upsert(db, path_key),
    }
  }

  /// Collect resolved thumbnail paths for a slice of book paths.
  pub fn collect_paths(&mut self, keys: &[SharedString], db: &DB) -> Vec<ThumbnailPath> {
    keys.iter().map(|k| self.get_or_resolve(db, k)).collect()
  }
}
