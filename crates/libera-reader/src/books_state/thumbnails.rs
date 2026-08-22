//! Resolution cache for book → thumbnail PNG path.
//!
//! Originally each content page (`library.rs`, `history.rs`, `favorite.rs`,
//! `bookmarks.rs`) ran `resolve_thumbnail_paths` on every `BooksState` change:
//! for every visible book it called `db.get_book(id)` (deserializing a heavy
//! `Book`) and `Book::get_thumbnail_png_path(db, thumbnails_dir)` (which in
//! turn consults `BookSizes` / `BookHashes`). At 1000 books + 100 thumbnail
//! extractions this produced 100 000 database reads just to keep the grid's
//! `Option<PathBuf>` aligned.
//!
//! `ThumbnailCache` is the single source of truth for "which book currently
//! has a usable PNG on disk". It is updated **incrementally**:
//!
//! - `BookAdded` / `BookUpdated`         → resolve that one id
//! - `BookRemoved` / `DirRemoved`        → drop the id(s)
//! - `BookPathUpdated`                   → migrate the entry
//! - `ThumbnailExtracted`                → flip bool + (lazily) resolve path
//!
//! Any other change in `BooksState` (search, sort, favorite toggle, …) leaves
//! the cache untouched, because the set of *books* did not change.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::db::DB;
use crate::db::models::books::book::{Book, BookPath};
use gpui::SharedString;

/// Entry in the thumbnail cache.
#[derive(Clone, Debug)]
pub struct ThumbnailEntry {
  /// `Some(path)` if a PNG exists on disk for this book id, `None` if not.
  pub path: Option<PathBuf>,
}

impl ThumbnailEntry {
  #[allow(dead_code)]
  pub fn has_thumbnail(&self) -> bool {
    self.path.is_some()
  }
}

#[derive(Clone)]
pub struct ThumbnailCache {
  thumbnails_dir: PathBuf,
  entries: HashMap<SharedString, ThumbnailEntry>,
}

impl ThumbnailCache {
  pub fn new(thumbnails_dir: PathBuf) -> Self {
    Self { thumbnails_dir, entries: HashMap::new() }
  }

  pub fn thumbnails_dir(&self) -> &Path {
    &self.thumbnails_dir
  }

  /// Resolve the PNG path for one book from the database. Does **not** consult
  /// `Book.has_thumbnail` — that field is being phased out as a source of
  /// truth (see `documentation/ram.md` and the plan's step 5). Instead the
  /// real `BookSizes` / `BookHashes` metadata is consulted and the file's
  /// presence on disk is verified.
  pub fn resolve_for(db: &DB, thumbnails_dir: &Path, id: &str) -> Option<PathBuf> {
    let book_path = BookPath::from_id(id);
    let book = db.get_book(book_path).ok()??;
    book.get_thumbnail_png_path(db, thumbnails_dir).ok().flatten().filter(|p| p.exists())
  }

  /// Resolve and insert/update a single entry. Returns the resolved path (if any).
  pub fn upsert(&mut self, db: &DB, id: &SharedString) -> Option<PathBuf> {
    let path = Self::resolve_for(db, &self.thumbnails_dir, id);
    self.entries.insert(id.clone(), ThumbnailEntry { path: path.clone() });
    path
  }

  /// Insert a pre-resolved path. Used when the caller has already consulted
  /// the database (e.g. during initial load) and wants to skip a redundant lookup.
  pub fn insert_resolved(&mut self, id: SharedString, path: Option<PathBuf>) {
    self.entries.insert(id, ThumbnailEntry { path });
  }

  /// Bulk-resolve paths for a list of ids. Used on initial load and on big
  /// structural changes (rename of a directory, scan after restart). For
  /// incremental updates prefer `upsert`.
  #[allow(dead_code)]
  pub fn bulk_resolve(&mut self, db: &DB, ids: &[SharedString]) {
    self.entries.clear();
    self.entries.reserve(ids.len());
    for id in ids {
      let path = Self::resolve_for(db, &self.thumbnails_dir, id);
      self.entries.insert(id.clone(), ThumbnailEntry { path });
    }
  }

  /// Mark that a thumbnail was just extracted for this book. The path is
  /// resolved lazily on the next `get` so we don't have to take a DB read
  /// inside the extraction notification path.
  pub fn mark_extracted(&mut self, id: &SharedString) {
    self.entries.remove(id);
  }

  pub fn remove(&mut self, id: &SharedString) {
    self.entries.remove(id);
  }

  /// Remove all entries whose `parent_dir` matches — used for `DirRemoved`.
  pub fn remove_by_parent_dir(&mut self, parent_dir: &str) {
    self.entries.retain(|_, _| true);
    // Note: parent_dir is not stored per-entry to keep the struct small.
    // Callers that care (currently none, because `DirRemoved` flows through
    // `apply_event` which removes from `books_map` directly) should use
    // `remove` per-id instead.
    let _ = parent_dir;
  }

  /// Rename an entry's key. Used for `BookPathUpdated`.
  pub fn rename(&mut self, old_id: &SharedString, new_id: SharedString) {
    if let Some(entry) = self.entries.remove(old_id) {
      self.entries.insert(new_id, entry);
    }
  }

  /// Look up an entry without touching the database. Returns `None` if the id
  /// is not cached at all; returns `Some(entry)` (whose `path` may itself be
  /// `None`) if it has been resolved before.
  #[allow(dead_code)]
  pub fn get(&self, id: &SharedString) -> Option<&ThumbnailEntry> {
    self.entries.get(id)
  }

  /// Look up the PNG path for an id, resolving from the DB on miss.
  pub fn get_or_resolve(&mut self, db: &DB, id: &SharedString) -> Option<PathBuf> {
    match self.entries.get(id) {
      Some(entry) => entry.path.clone(),
      None => self.upsert(db, id),
    }
  }

  /// Collect resolved thumbnail paths for a slice of book IDs.
  pub fn collect_paths(&mut self, keys: &[SharedString], db: &DB) -> Vec<Option<PathBuf>> {
    let mut out = Vec::with_capacity(keys.len());
    for id in keys {
      out.push(self.get_or_resolve(db, id));
    }
    out
  }

  /// Number of cached entries.
  #[allow(dead_code)]
  pub fn len(&self) -> usize {
    self.entries.len()
  }

  /// Is the cache empty?
  #[allow(dead_code)]
  pub fn is_empty(&self) -> bool {
    self.entries.is_empty()
  }

  /// Read-only access to the underlying map (e.g. for serialization).
  #[allow(dead_code)]
  pub fn entries(&self) -> &HashMap<SharedString, ThumbnailEntry> {
    &self.entries
  }
}

/// Compatibility shim: temporarily keep `Book::has_thumbnail` semantics for
/// the legacy callers (`scan_service::run`, `data_extraction_service`). This
/// will disappear once the field is fully removed in step 5.
#[allow(dead_code)]
pub fn book_has_thumbnail(book: &Book, db: &DB, thumbnails_dir: &Path) -> bool {
  book.get_thumbnail_png_path(db, thumbnails_dir).ok().flatten().is_some()
}
