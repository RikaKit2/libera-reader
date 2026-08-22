pub mod models;
pub mod search;
pub mod sort;
pub mod thumbnails;

use crate::db::DB;
use crate::db::models::books::book::{Book, BookDir, BookPath};
use gpui::{AsyncApp, Context, SharedString, WeakEntity};
use std::collections::HashMap as StdHashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};
use thumbnails::ThumbnailCache;
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

pub use models::{SortConfig, SortField, TargetList};

pub static BOOKS_STATE: OnceLock<BooksState> = OnceLock::new();

/// Thread-safe in-memory state for all books.
/// Wraps `BooksStateData` in an `Arc<RwLock<...>>` for safe multi-threaded access
/// by background Tokio tasks (scanner, notify watcher, extractor) and GPUI UI.
#[derive(Clone)]
pub struct BooksState {
  inn: Arc<RwLock<BooksStateData>>,
  notify_tx: Arc<Mutex<Option<UnboundedSender<()>>>>,
}

impl gpui::Global for BooksState {}

#[derive(Clone)]
pub struct BooksStateEntity(pub gpui::Entity<BooksState>);

impl gpui::Global for BooksStateEntity {}
pub struct BooksStateData {
  /// Map from book id (full path string) to Book
  pub books_map: StdHashMap<SharedString, Book>,

  /// Single source of truth for which book currently has a usable PNG on disk.
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

  pub search_generation: StdHashMap<TargetList, u64>,
}

impl BooksStateData {
  pub fn from_db(thumbnails_dir: PathBuf, db: &DB) -> Self {
    let mut thumbnails = ThumbnailCache::new(thumbnails_dir);
    let mut books_map = StdHashMap::new();

    let thumbnails_dir_ref = thumbnails.thumbnails_dir().to_path_buf();
    let _ = db.for_each_book(|book| {
      let id: SharedString = book.id.clone().into();
      let path = ThumbnailCache::resolve_for(db, &thumbnails_dir_ref, &book.id);
      thumbnails.insert_resolved(id.clone(), path);
      books_map.insert(id, book);
      Ok(())
    });

    let mut data = Self {
      books_map,
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
      search_generation: StdHashMap::new(),
    };

    data.rebuild_all();
    data
  }

  pub fn rebuild_all(&mut self) {
    self.rebuild_and_sort(TargetList::Library);
    self.rebuild_and_sort(TargetList::Favorites);
    self.rebuild_and_sort(TargetList::History);
    self.rebuild_and_sort(TargetList::Bookmarks);
  }

  pub fn upsert_book(&mut self, book: Book) {
    let id: SharedString = book.id.clone().into();
    self.books_map.insert(id, book);
  }

  pub fn remove_book(&mut self, path: &BookPath) {
    let id: SharedString = path.full_path_string();
    self.books_map.remove(&id);
    self.thumbnails.remove(&id);
  }

  pub fn update_book_path(&mut self, old_path: &BookPath, new_path: &BookPath) {
    let old_id: SharedString = old_path.full_path_string();
    let new_id: SharedString = new_path.full_path_string();
    if let Some(book) = self.books_map.remove(&old_id) {
      let mut updated = book;
      updated.id = new_id.to_string();
      updated.parent_dir = new_path.parent_dir.full_path().to_string();
      updated.book_path = new_path.clone();
      self.books_map.insert(new_id.clone(), updated);
    }
    self.thumbnails.rename(&old_id, new_id);
  }

  pub fn remove_dir(&mut self, dir: &BookDir) {
    let dir_path = dir.full_path().to_string();
    self.books_map.retain(|_id, book| book.parent_dir != dir_path);
    self.thumbnails.remove_by_parent_dir(&dir_path);
  }

  pub fn mark_thumbnail_extracted(&mut self, path: &BookPath) {
    let id: SharedString = path.full_path_string();
    self.thumbnails.mark_extracted(&id);
  }
}

impl BooksState {
  pub fn new(thumbnails_dir: PathBuf, db: &DB) -> Self {
    let state = Self {
      inn: Arc::new(RwLock::new(BooksStateData::from_db(thumbnails_dir, db))),
      notify_tx: Arc::new(Mutex::new(None)),
    };
    let _ = BOOKS_STATE.set(state.clone());
    state
  }

  pub fn global() -> Option<&'static BooksState> {
    BOOKS_STATE.get()
  }

  pub fn read(&self) -> RwLockReadGuard<'_, BooksStateData> {
    self.inn.read().unwrap()
  }

  pub fn write(&self) -> RwLockWriteGuard<'_, BooksStateData> {
    self.inn.write().unwrap()
  }

  /// Attach GPUI UI notification listener when creating the root GPUI entity.
  /// Spawns a background listener on the GPUI UI thread that coalesces rapid updates
  /// into a single `cx.notify()` call to avoid redundant frame rendering.
  pub fn attach_ui(&self, cx: &mut Context<Self>) -> Self {
    let (tx, mut rx) = unbounded_channel::<()>();
    *self.notify_tx.lock().unwrap() = Some(tx);

    cx.spawn(|this: WeakEntity<BooksState>, cx: &mut AsyncApp| {
      let mut owned_cx = cx.clone();
      async move {
        while rx.recv().await.is_some() {
          // Drain all pending notifications to coalesce into a single UI redraw
          while rx.try_recv().is_ok() {}
          let _ = this.update(&mut owned_cx, |_, cx| {
            cx.notify();
          });
        }
      }
    })
    .detach();

    self.clone()
  }

  pub fn notify(&self) {
    if let Some(tx) = self.notify_tx.lock().unwrap().as_ref() {
      let _ = tx.send(());
    }
  }

  // --- Direct mutation methods for background services and UI ---

  pub fn add_book(&self, book: Book) {
    {
      let mut data = self.write();
      data.upsert_book(book);
      data.rebuild_all();
    }
    self.notify();
  }

  pub fn add_books_batch(&self, books: &[Book]) {
    if books.is_empty() {
      return;
    }
    {
      let mut data = self.write();
      for book in books {
        data.upsert_book(book.clone());
      }
      data.rebuild_all();
    }
    self.notify();
  }

  pub fn update_book(&self, book: &Book) {
    {
      let mut data = self.write();
      data.upsert_book(book.clone());
      data.rebuild_all();
    }
    self.notify();
  }
  pub fn remove_book(&self, path: &BookPath) {
    {
      let mut data = self.write();
      data.remove_book(path);
      data.rebuild_all();
    }
    self.notify();
  }

  pub fn remove_books(&self, paths: &[BookPath]) {
    if paths.is_empty() {
      return;
    }
    {
      let mut data = self.write();
      for path in paths {
        data.remove_book(path);
      }
      data.rebuild_all();
    }
    self.notify();
  }

  pub fn update_book_path(&self, old_path: &BookPath, new_path: &BookPath) {
    {
      let mut data = self.write();
      data.update_book_path(old_path, new_path);
      data.rebuild_all();
    }
    self.notify();
  }

  pub fn remove_dir(&self, dir: &BookDir) {
    {
      let mut data = self.write();
      data.remove_dir(dir);
      data.rebuild_all();
    }
    self.notify();
  }

  pub fn mark_thumbnail_extracted(&self, path: &BookPath) {
    {
      let mut data = self.write();
      data.mark_thumbnail_extracted(path);
    }
    self.notify();
  }

  // --- Convenience helper methods for UI ---

  pub fn library_keys(&self) -> Vec<SharedString> {
    self.read().library_keys.clone()
  }

  pub fn favorites_keys(&self) -> Vec<SharedString> {
    self.read().favorites_keys.clone()
  }

  pub fn history_keys(&self) -> Vec<SharedString> {
    self.read().history_keys.clone()
  }

  pub fn bookmarks_keys(&self) -> Vec<SharedString> {
    self.read().bookmarks_keys.clone()
  }

  pub fn total_books(&self, target: TargetList) -> usize {
    let data = self.read();
    match target {
      TargetList::Library => data.library_keys.len(),
      TargetList::Favorites => data.favorites_keys.len(),
      TargetList::History => data.history_keys.len(),
      TargetList::Bookmarks => data.bookmarks_keys.len(),
    }
  }

  pub fn sort_field(&self, target: TargetList) -> SortField {
    self.read().sort_config(target).field
  }

  pub fn is_reversed(&self, target: TargetList) -> bool {
    self.read().sort_config(target).is_reversed
  }

  pub fn collect_thumbnail_paths(&self, keys: &[SharedString], db: &DB) -> Vec<Option<PathBuf>> {
    self.write().thumbnails.collect_paths(keys, db)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn make_test_state() -> BooksState {
    BooksState {
      inn: Arc::new(RwLock::new(BooksStateData {
        books_map: StdHashMap::new(),
        thumbnails: ThumbnailCache::new(PathBuf::from("/tmp")),
        library_keys: Vec::new(),
        favorites_keys: Vec::new(),
        history_keys: Vec::new(),
        bookmarks_keys: Vec::new(),
        library_sort: SortConfig::default(),
        favorites_sort: SortConfig::default(),
        history_sort: SortConfig::default(),
        bookmarks_sort: SortConfig::default(),
        library_search: "".into(),
        favorites_search: "".into(),
        history_search: "".into(),
        bookmarks_search: "".into(),
        search_generation: StdHashMap::new(),
      })),
      notify_tx: Arc::new(Mutex::new(None)),
    }
  }

  #[test]
  fn test_books_state_direct_mutations() {
    let state = make_test_state();
    use crate::db::models::UserData;
    use crate::db::models::books::book::BookSize;

    let book1 = Book {
      id: "/path/to/book1.pdf".to_string(),
      parent_dir: "/path/to".to_string(),
      book_path: BookPath::new(std::path::Path::new("/path/to/book1.pdf")).unwrap(),
      book_size: BookSize::BYTES(1024),
      user_data: UserData { favorite: false, last_opened: 0 },
      bookmark_count: 0,
    };

    let book2 = Book {
      id: "/path/to/book2.epub".to_string(),
      parent_dir: "/path/to".to_string(),
      book_path: BookPath::new(std::path::Path::new("/path/to/book2.epub")).unwrap(),
      book_size: BookSize::BYTES(2048),
      user_data: UserData { favorite: true, last_opened: 100 },
      bookmark_count: 1,
    };
    state.add_books_batch(&[book1.clone(), book2.clone()]);
    assert_eq!(state.total_books(TargetList::Library), 2);
    assert_eq!(state.total_books(TargetList::Favorites), 1);
    assert_eq!(state.total_books(TargetList::History), 1);
    assert_eq!(state.total_books(TargetList::Bookmarks), 1);

    // 2. Mark thumbnail extracted
    let bp1 = BookPath::new(std::path::Path::new("/path/to/book1.pdf")).unwrap();
    state.mark_thumbnail_extracted(&bp1);
    // 3. Update book
    let mut updated1 = book1.clone();
    updated1.user_data.favorite = true;
    state.update_book(&updated1);

    // 4. Remove book
    state.remove_book(&bp1);
    assert_eq!(state.total_books(TargetList::Library), 1);
    assert_eq!(state.total_books(TargetList::Favorites), 1);

    // 5. Remove dir
    let dir = BookDir::new(PathBuf::from("/path/to"));
    state.remove_dir(&dir);
    assert_eq!(state.total_books(TargetList::Library), 0);
  }
}
