pub mod events;
pub mod models;
pub mod search;
pub mod sort;
pub mod thumbnails;

use gpui::{Context, SharedString, Task};
use libera_reader_core::db::models::books::book::{Book, BookPath, BookSnapshot};
use libera_reader_core::types::LibraryEvent;
use std::collections::HashMap as StdHashMap;
use std::path::PathBuf;
use thumbnails::ThumbnailCache;
use tokio::sync::broadcast::{
  Receiver,
  error::RecvError::{Closed, Lagged},
};

pub use models::{LightBook, SortConfig, SortField, TargetList};

/// Reconstruct a `BookPath` from a snapshot's `id` for the extraction queue.
///
/// This avoids round-tripping the heavy `Book` through the extraction channel
/// when the only thing the extractor needs is the path.
pub(crate) fn snapshot_to_book_path(snapshot: &BookSnapshot) -> BookPath {
  BookPath::from_id(&snapshot.id)
}

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
    thumbnails_dir: PathBuf, db: &libera_reader_core::db::DB, mut event_rx: Receiver<LibraryEvent>,
    cx: &mut Context<Self>,
  ) -> Self {
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

    state.rebuild_and_sort(TargetList::Library);
    state.rebuild_and_sort(TargetList::Favorites);
    state.rebuild_and_sort(TargetList::History);
    state.rebuild_and_sort(TargetList::Bookmarks);

    cx.spawn(|this: gpui::WeakEntity<BooksState>, cx: &mut gpui::AsyncApp| {
      let mut owned_cx = cx.clone();
      async move {
        loop {
          match event_rx.recv().await {
            Ok(event) => {
              let _ = this.update(&mut owned_cx, |state, context| {
                state.apply_event(event, context);
                context.notify();
              });
            }
            Err(Lagged(_)) => {
              continue;
            }
            Err(Closed) => {
              break;
            }
          }
        }
      }
    })
    .detach();

    state
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
}
