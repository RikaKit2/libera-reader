pub mod events;
pub mod models;
pub mod search;
pub mod sort;

use gpui::{Context, SharedString, Task};
use libera_reader_core::db::models::books::Books;
use libera_reader_core::db::models::books::book::{Book, BookDir, BookPath};
use libera_reader_core::types::{HashMap, LibraryEvent};
use std::cell::RefCell;
use std::collections::HashMap as StdHashMap;
use std::rc::Rc;
use tokio::sync::broadcast::Receiver;

pub use models::{SortConfig, SortField, TargetList};

pub struct BooksState {
  pub books_map: HashMap<BookDir, Books>,

  pub library_keys: Vec<BookPath>,
  pub favorites_keys: Vec<BookPath>,
  pub history_keys: Vec<BookPath>,
  pub bookmarks_keys: Vec<BookPath>,

  pub library_sort: SortConfig,
  pub favorites_sort: SortConfig,
  pub history_sort: SortConfig,
  pub bookmarks_sort: SortConfig,

  pub library_search: SharedString,
  pub favorites_search: SharedString,
  pub history_search: SharedString,
  pub bookmarks_search: SharedString,

  pub search_tasks: StdHashMap<TargetList, Task<()>>,

  /// Global cover cache shared with BooksGrid.
  /// When a thumbnail is generated, BookUpdated clears the entry so next render re-checks.
  pub cover_cache: Rc<RefCell<HashMap<BookPath, bool>>>,
}

impl BooksState {
  pub fn new(
    initial_books: HashMap<BookDir, Books>, mut event_rx: Receiver<LibraryEvent>,
    cx: &mut Context<Self>,
  ) -> Self {
    let mut state = Self {
      books_map: initial_books,
      library_keys: Vec::new(),
      favorites_keys: Vec::new(),
      history_keys: Vec::new(),
      bookmarks_keys: Vec::new(),
      library_sort: SortConfig::default(),
      favorites_sort: SortConfig::default(),

      // For history, by default we sort by opening date, new ones on top (is_reversed: true)
      history_sort: SortConfig { field: SortField::LastOpened, is_reversed: true },

      bookmarks_sort: SortConfig::default(),
      library_search: "".into(),
      favorites_search: "".into(),
      history_search: "".into(),
      bookmarks_search: "".into(),
      search_tasks: StdHashMap::new(),
      cover_cache: Rc::new(RefCell::new(HashMap::default())),
    };

    state.rebuild_and_sort(TargetList::Library);
    state.rebuild_and_sort(TargetList::Favorites);
    state.rebuild_and_sort(TargetList::History);
    state.rebuild_and_sort(TargetList::Bookmarks);

    cx.spawn(|this: gpui::WeakEntity<BooksState>, cx: &mut gpui::AsyncApp| {
      let mut owned_cx = cx.clone();
      async move {
        while let Ok(event) = event_rx.recv().await {
          let _ = this.update(&mut owned_cx, |state, context| {
            state.apply_event(event, context);
            context.notify();
          });
        }
      }
    })
    .detach();

    state
  }

  pub fn get_book(&self, path: &BookPath) -> Option<&Book> {
    let key = path.file_name();
    self.books_map.get(&path.parent_dir).and_then(|dir_books| dir_books.storage.get(&key))
  }
}
