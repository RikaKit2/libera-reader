pub mod models;
pub mod search;
pub mod sort;
pub mod thumbnails;

use crate::db::models::books::book::{Book, BookDir, BookPath, BookSnapshot};
use gpui::{AsyncApp, Context, SharedString, Task, WeakEntity};
use std::collections::HashMap as StdHashMap;
use std::path::PathBuf;
use thumbnails::ThumbnailCache;
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

pub use models::{LightBook, SortConfig, SortField, TargetList};

/// In-memory state for all books.
/// No heavy `Book` structs — only lightweight `LightBook`s.
pub struct BooksState {
  /// Map from book id (full path string) to LightBook
  pub books_map: StdHashMap<SharedString, LightBook>,

  /// Single source of truth for which book currently has a usable PNG on disk.
  /// Incrementally updated from `apply_event`; queried by every content page
  /// instead of doing per-frame `db.get_book()` calls.
  pub thumbnails: ThumbnailCache,

  pub library_keys: Vec<SharedString>,
  pub favorites_keys: Vec<SharedString>,
  pub history_keys: Vec<SharedString>,
  pub bookmarks_keys: Vec<SharedString>,

  pub library_sort: SortConfig,
  pub favorites_sort: SortConfig,
  pub history_sort: SortConfig,
  pub bookmarks_sort: SortConfig,

  pub library_search: SharedString,
  pub favorites_search: SharedString,
  pub history_search: SharedString,
  pub bookmarks_search: SharedString,

  #[allow(dead_code)]
  pub search_tasks: StdHashMap<TargetList, Task<()>>,

  /// Generation counter for search debounce: each new search increments the generation
  /// for its target. When the debounced task fires, it checks if its generation is still
  /// current; if not, it skips the rebuild. This effectively cancels stale search tasks.
  #[allow(dead_code)]
  pub search_generation: StdHashMap<TargetList, u64>,
}

impl BooksState {
  pub fn new(
    thumbnails_dir: PathBuf, db: &crate::db::DB, cx: &mut Context<Self>,
  ) -> (Self, BooksStateHandle) {
    let thumbnails = ThumbnailCache::new(thumbnails_dir);

    let mut state = Self {
      books_map: StdHashMap::new(),
      thumbnails,
      library_keys: Vec::new(),
      favorites_keys: Vec::new(),
      history_keys: Vec::new(),
      bookmarks_keys: Vec::new(),
      library_sort: SortConfig::default(),
      favorites_sort: SortConfig::default(),
      history_sort: SortConfig { field: SortField::LastOpened, is_reversed: true },
      bookmarks_sort: SortConfig::default(),
      library_search: "".into(),
      favorites_search: "".into(),
      history_search: "".into(),
      bookmarks_search: "".into(),
      search_tasks: StdHashMap::new(),
      search_generation: StdHashMap::new(),
    };

    // Stream books one-by-one so we never hold the full `Vec<Book>` in memory.
    // Each book is converted to a LightBook immediately and the heavy `Book`
    // is dropped before the next one is read.
    let thumbnails_dir = state.thumbnails.thumbnails_dir().to_path_buf();
    let _ = db.for_each_book(|book| {
      let id: SharedString = book.id.clone().into();
      let path = ThumbnailCache::resolve_for(db, &thumbnails_dir, &book.id);
      let has_thumbnail = path.is_some();
      state.thumbnails.insert_resolved(id.clone(), path);
      state.books_map.insert(id, LightBook::from_book(&book, has_thumbnail));
      Ok(())
    });

    state.rebuild_all();

    let (tx, mut rx) = unbounded_channel::<BooksUpdate>();
    cx.spawn(|this: WeakEntity<BooksState>, cx: &mut AsyncApp| {
      let mut owned_cx = cx.clone();
      async move {
        while let Some(update) = rx.recv().await {
          let _ = this.update(&mut owned_cx, |state, cx| {
            state.apply_update(update);
            cx.notify();
          });
        }
      }
    })
    .detach();

    let handle = BooksStateHandle::new(tx);
    (state, handle)
  }
  #[allow(dead_code)]
  pub fn get_light_book(&self, id: &SharedString) -> Option<&LightBook> {
    self.books_map.get(id)
  }

  /// Insert or update a LightBook from a full Book.
  ///
  /// Used by the initial-load path in `BooksState::new`. Event-driven updates
  /// go through `upsert_snapshot` after services convert `Book` to `BookSnapshot`.
  #[allow(dead_code)]
  pub(crate) fn upsert_book(&mut self, book: &Book) {
    let id: SharedString = book.id.clone().into();
    let has_thumbnail = self.books_map.get(&id).is_some_and(|lb| lb.has_thumbnail);
    let light = LightBook::from_book(book, has_thumbnail);
    self.books_map.insert(id, light);
  }

  /// Insert or update a LightBook from a UI-ready snapshot.
  ///
  /// This is the post-`BookSnapshot` path used by `apply_event` for the
  /// `BookAdded` / `BooksBatchAdded` / `BookUpdated` variants. The snapshot
  /// already carries the resolved `has_thumbnail` flag from the service side,
  /// so we preserve the existing value only when the snapshot's flag is stale
  /// (e.g. the UI already saw a `ThumbnailExtracted` event ahead of the
  /// snapshot arriving).
  pub(crate) fn upsert_snapshot(&mut self, snapshot: &BookSnapshot) {
    let id: SharedString = snapshot.id.clone().into();
    let has_thumbnail =
      snapshot.has_thumbnail || self.books_map.get(&id).is_some_and(|lb| lb.has_thumbnail);
    let light = LightBook::from_snapshot(snapshot, has_thumbnail);
    self.books_map.insert(id, light);
  }

  /// Insert a LightBook with existing has_thumbnail state
  pub(crate) fn upsert_light(&mut self, id: SharedString, light: LightBook) {
    self.books_map.insert(id, light);
  }

  /// Rebuild and sort all four target lists.
  pub fn rebuild_all(&mut self) {
    self.rebuild_and_sort(TargetList::Library);
    self.rebuild_and_sort(TargetList::Favorites);
    self.rebuild_and_sort(TargetList::History);
    self.rebuild_and_sort(TargetList::Bookmarks);
  }

  /// Add a single book snapshot to state and rebuild lists.
  pub fn add_book(&mut self, snapshot: &BookSnapshot) {
    self.upsert_snapshot(snapshot);
    self.rebuild_all();
  }

  /// Add a batch of book snapshots to state and rebuild lists once.
  pub fn add_books_batch(&mut self, snapshots: &[BookSnapshot]) {
    for snapshot in snapshots {
      self.upsert_snapshot(snapshot);
    }
    self.rebuild_all();
  }

  /// Update an existing book snapshot and rebuild lists.
  pub fn update_book(&mut self, snapshot: &BookSnapshot) {
    self.upsert_snapshot(snapshot);
    self.rebuild_all();
  }

  /// Remove a book from state and thumbnail cache, then rebuild lists.
  pub fn remove_book(&mut self, path: &BookPath) {
    let id: SharedString = path.full_path_string();
    self.books_map.remove(&id);
    self.thumbnails.remove(&id);
    self.rebuild_all();
  }

  /// Update a book's path/name/dir after rename or move.
  pub fn update_book_path(&mut self, old_path: &BookPath, new_path: &BookPath) {
    let old_id: SharedString = old_path.full_path_string();
    let new_id: SharedString = new_path.full_path_string();
    if let Some(light) = self.books_map.remove(&old_id) {
      let mut updated = light;
      updated.id = new_id.clone();
      updated.parent_dir = new_path.parent_dir.full_path();
      updated.name = new_path.name.clone();
      self.upsert_light(new_id.clone(), updated);
    }
    self.thumbnails.rename(&old_id, new_id);
    self.rebuild_all();
  }

  /// Remove all books belonging to a deleted directory.
  pub fn remove_dir(&mut self, dir: &BookDir) {
    let dir_path = dir.full_path().to_string();
    self.books_map.retain(|_id, light| light.parent_dir.as_ref() != dir_path);
    self.thumbnails.remove_by_parent_dir(&dir_path);
    self.rebuild_all();
  }

  /// Mark that a thumbnail has been extracted for the book.
  pub fn mark_thumbnail_extracted(&mut self, path: &BookPath) {
    let id: SharedString = path.full_path_string();
    if let Some(light) = self.books_map.get_mut(&id) {
      light.has_thumbnail = true;
    }
    self.thumbnails.mark_extracted(&id);
  }

  /// Apply a state update command dispatched from a background service.
  pub fn apply_update(&mut self, update: BooksUpdate) {
    match update {
      BooksUpdate::AddBooks(snapshots) => self.add_books_batch(&snapshots),
      BooksUpdate::AddBook(snapshot) => self.add_book(&snapshot),
      BooksUpdate::UpdateBook(snapshot) => self.update_book(&snapshot),
      BooksUpdate::RemoveBook(path) => self.remove_book(&path),
      BooksUpdate::UpdateBookPath { old_path, new_path } => {
        self.update_book_path(&old_path, &new_path);
      }
      BooksUpdate::RemoveDir(dir) => self.remove_dir(&dir),
      BooksUpdate::ThumbnailExtracted(path) => self.mark_thumbnail_extracted(&path),
    }
  }
}

/// Commands dispatched from background services to update BooksState on the UI thread.
#[derive(Debug, Clone)]
pub enum BooksUpdate {
  AddBooks(Vec<BookSnapshot>),
  AddBook(BookSnapshot),
  UpdateBook(BookSnapshot),
  RemoveBook(BookPath),
  UpdateBookPath { old_path: BookPath, new_path: BookPath },
  RemoveDir(BookDir),
  ThumbnailExtracted(BookPath),
}

/// Thread-safe handle to dispatch BooksState updates from background Tokio tasks
/// to GPUI's main thread.
#[derive(Clone)]
pub struct BooksStateHandle {
  tx: UnboundedSender<BooksUpdate>,
}

impl BooksStateHandle {
  pub fn new(tx: UnboundedSender<BooksUpdate>) -> Self {
    Self { tx }
  }

  pub fn add_book(&self, snapshot: BookSnapshot) {
    let _ = self.tx.send(BooksUpdate::AddBook(snapshot));
  }

  pub fn add_books_batch(&self, snapshots: Vec<BookSnapshot>) {
    let _ = self.tx.send(BooksUpdate::AddBooks(snapshots));
  }

  pub fn update_book(&self, snapshot: BookSnapshot) {
    let _ = self.tx.send(BooksUpdate::UpdateBook(snapshot));
  }

  pub fn remove_book(&self, path: BookPath) {
    let _ = self.tx.send(BooksUpdate::RemoveBook(path));
  }

  pub fn update_book_path(&self, old_path: BookPath, new_path: BookPath) {
    let _ = self.tx.send(BooksUpdate::UpdateBookPath { old_path, new_path });
  }

  pub fn remove_dir(&self, dir: BookDir) {
    let _ = self.tx.send(BooksUpdate::RemoveDir(dir));
  }

  pub fn mark_thumbnail_extracted(&self, path: BookPath) {
    let _ = self.tx.send(BooksUpdate::ThumbnailExtracted(path));
  }
}
