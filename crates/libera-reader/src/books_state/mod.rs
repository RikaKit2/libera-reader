pub mod models;
pub mod search;
pub mod sort;
pub mod thumbnails;

use crate::db::DB;
use crate::db::models::books::book::{Book, BookDir, BookPath};
use crate::types::HashMap;
use gpui::{AsyncApp, Context, SharedString, WeakEntity};
pub use models::{SortConfig, SortField, TargetList};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};
use thumbnails::{ThumbnailCache, ThumbnailPath};
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

/// Map of book full path to its loaded Book entity.
pub type BooksMap = HashMap<SharedString, Book>;

/// List of book full paths representing an ordered collection (e.g. library, history, favorites).
pub type BookKeys = Vec<SharedString>;
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
  pub books_map: BooksMap,

  /// Single source of truth for which book currently has a usable PNG on disk.
  pub thumbnails: ThumbnailCache,

  pub library_keys: BookKeys,
  pub favorites_keys: BookKeys,
  pub history_keys: BookKeys,
  pub bookmarks_keys: BookKeys,

  pub library_sort: SortConfig,
  pub favorites_sort: SortConfig,
  pub history_sort: SortConfig,
  pub bookmarks_sort: SortConfig,

  pub library_search: SharedString,
  pub favorites_search: SharedString,
  pub history_search: SharedString,
  pub bookmarks_search: SharedString,

  pub search_generation: HashMap<TargetList, u64>,
}

impl BooksStateData {
  pub fn load(thumbnails_dir: PathBuf, db: &DB) -> Self {
    let mut thumbnails = ThumbnailCache::new(thumbnails_dir);
    let mut books_map = BooksMap::default();

    let thumbnails_dir_ref = thumbnails.thumbnails_dir().to_path_buf();
    let _ = Book::for_each(db, |book| {
      let path_key = book.book_path.full_path_string();
      let path = ThumbnailCache::resolve_for(db, &thumbnails_dir_ref, &book.book_path);
      thumbnails.insert_resolved(path_key.clone(), path);
      books_map.insert(path_key, book);
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
      search_generation: HashMap::default(),
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
    let path_key = book.book_path.full_path_string();
    self.books_map.insert(path_key, book);
  }

  pub fn remove_book(&mut self, path: &BookPath) {
    let path_key = path.full_path_string();
    self.books_map.swap_remove(&path_key);
    self.thumbnails.remove(&path_key);
  }

  pub fn update_book_path(&mut self, old_path: &BookPath, new_path: &BookPath) {
    let old_key = old_path.full_path_string();
    let new_key = new_path.full_path_string();
    if let Some(book) = self.books_map.swap_remove(&old_key) {
      let mut updated = book;
      updated.parent_dir = new_path.parent_dir.clone();
      updated.book_path = new_path.clone();
      self.books_map.insert(new_key.clone(), updated);
    }
    self.thumbnails.rename(&old_key, new_key);
  }

  pub fn remove_dir(&mut self, dir: &BookDir) {
    self.books_map.retain(|path_key, book| {
      let keep = &book.parent_dir != dir;
      if !keep {
        self.thumbnails.remove(path_key);
      }
      keep
    });
  }

  pub fn mark_thumbnail_extracted(&mut self, path: &BookPath) {
    let path_key = path.full_path_string();
    self.thumbnails.remove(&path_key);
  }
}

impl BooksState {
  /// Create `BooksState` by pulling required dependencies directly from GPUI `App`.
  pub fn new(cx: &gpui::App) -> Self {
    use crate::app_ext::AppExt;
    let thumbnails_dir = cx.app_dirs().read().thumbnails_dir.clone();
    let db = cx.db();
    Self::from_deps(thumbnails_dir, db)
  }

  /// Create `BooksState` directly from explicit dependencies (used in tests and benchmarks).
  pub fn from_deps(thumbnails_dir: PathBuf, db: &DB) -> Self {
    Self {
      inn: Arc::new(RwLock::new(BooksStateData::load(thumbnails_dir, db))),
      notify_tx: Arc::new(Mutex::new(None)),
    }
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

  pub fn library_keys(&self) -> BookKeys {
    self.read().library_keys.clone()
  }

  pub fn favorites_keys(&self) -> BookKeys {
    self.read().favorites_keys.clone()
  }

  pub fn history_keys(&self) -> BookKeys {
    self.read().history_keys.clone()
  }

  pub fn bookmarks_keys(&self) -> BookKeys {
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

  pub fn collect_thumbnail_paths(&self, keys: &[SharedString], db: &DB) -> Vec<ThumbnailPath> {
    self.write().thumbnails.collect_paths(keys, db)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn make_test_state() -> BooksState {
    BooksState {
      inn: Arc::new(RwLock::new(BooksStateData {
        books_map: BooksMap::default(),
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
        search_generation: HashMap::default(),
      })),
      notify_tx: Arc::new(Mutex::new(None)),
    }
  }

  #[test]
  fn test_books_state_direct_mutations() {
    let state = make_test_state();
    use crate::db::models::UserData;
    use crate::db::models::books::book::BookSize;

    let bp1 = BookPath::new(std::path::Path::new("/path/to/book1.pdf")).unwrap();
    let book1 = Book {
      parent_dir: bp1.parent_dir.clone(),
      book_path: bp1.clone(),
      book_size: BookSize::BYTES(1024),
      user_data: UserData { favorite: false, last_opened: 0 },
      bookmark_count: 0,
    };

    let bp2 = BookPath::new(std::path::Path::new("/path/to/book2.epub")).unwrap();
    let book2 = Book {
      parent_dir: bp2.parent_dir.clone(),
      book_path: bp2.clone(),
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
