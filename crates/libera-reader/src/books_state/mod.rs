pub mod events;
pub mod models;
pub mod search;
pub mod sort;

use gpui::{Context, SharedString, Task};
use libera_reader_core::db::models::books::book::Book;
use libera_reader_core::types::LibraryEvent;
use std::cell::RefCell;
use std::collections::HashMap as StdHashMap;
use std::rc::Rc;
use tokio::sync::broadcast::{
  Receiver,
  error::RecvError::{Closed, Lagged},
};

pub use models::{LightBook, SortConfig, SortField, TargetList};

/// In-memory state for all books.
/// No heavy `Book` structs — only lightweight `LightBook`s.
pub struct BooksState {
  /// Map from book id (full path string) to LightBook
  pub books_map: StdHashMap<SharedString, LightBook>,

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

  pub search_tasks: StdHashMap<TargetList, Task<()>>,

  /// Cache to track which books have thumbnails
  pub cover_cache: Rc<RefCell<StdHashMap<SharedString, bool>>>,
}

impl BooksState {
  pub fn new(
    initial_books: Vec<Book>, mut event_rx: Receiver<LibraryEvent>, cx: &mut Context<Self>,
  ) -> Self {
    let mut state = Self {
      books_map: StdHashMap::new(),
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
      cover_cache: Rc::new(RefCell::new(StdHashMap::new())),
    };

    // Insert initial books
    for book in initial_books {
      let id: SharedString = book.id.clone().into();
      let has_thumbnail = book.has_thumbnail;
      if has_thumbnail {
        state.cover_cache.borrow_mut().insert(id.clone(), true);
      }
      state.books_map.insert(id, LightBook::from_book(&book, has_thumbnail));
    }

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

  pub fn get_light_book(&self, id: &SharedString) -> Option<&LightBook> {
    self.books_map.get(id)
  }

  /// Insert or update a LightBook from a full Book
  pub(crate) fn upsert_book(&mut self, book: &Book) {
    let id: SharedString = book.id.clone().into();
    let has_thumbnail = self.books_map.get(&id).is_some_and(|lb| lb.has_thumbnail);
    let light = LightBook::from_book(book, has_thumbnail);
    self.books_map.insert(id, light);
  }

  /// Insert a LightBook with existing has_thumbnail state
  pub(crate) fn upsert_light(&mut self, id: SharedString, light: LightBook) {
    self.books_map.insert(id, light);
  }
}
