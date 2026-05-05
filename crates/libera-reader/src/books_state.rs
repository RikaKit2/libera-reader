#![allow(dead_code)]
#![allow(unused_imports)]

use gpui::*;
use gpui_component::select::SelectItem;
use rust_i18n::t;
use std::cmp::Ordering;
use std::collections::HashMap as StdHashMap;
use std::fmt;
use std::time::Duration;
use tokio::sync::broadcast;

use libera_reader_core::db::models::books::Books;
use libera_reader_core::db::models::books::book::{Book, BookDir, BookPath, BookSize};
use libera_reader_core::types::{HashMap, LibraryEvent};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SortField {
  Name,
  Size,
  Type,
}

impl SortField {
  pub fn all() -> &'static [Self] {
    &[Self::Name, Self::Size, Self::Type]
  }
}

impl fmt::Display for SortField {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      SortField::Name => write!(f, "Name"),
      SortField::Size => write!(f, "Size"),
      SortField::Type => write!(f, "Type"),
    }
  }
}

impl SelectItem for SortField {
  type Value = SortField;

  fn title(&self) -> SharedString {
    match self {
      SortField::Name => t!("components.sort_dropdown.fields.name").to_string().into(),
      SortField::Size => t!("components.sort_dropdown.fields.size").to_string().into(),
      SortField::Type => t!("components.sort_dropdown.fields.type").to_string().into(),
    }
  }

  fn value(&self) -> &Self::Value {
    self
  }
}

#[derive(Clone, Copy)]
pub struct SortConfig {
  pub field: SortField,
  pub is_reversed: bool,
}

impl Default for SortConfig {
  fn default() -> Self {
    Self { field: SortField::Name, is_reversed: false }
  }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum TargetList {
  Library,
  Favorites,
  History,
  Bookmarks,
}

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

  // Search queries
  pub library_search: SharedString,
  pub favorites_search: SharedString,
  pub history_search: SharedString,
  pub bookmarks_search: SharedString,

  // Tasks used to cancel previous debounce timers
  pub search_tasks: StdHashMap<TargetList, Task<()>>,
}

impl BooksState {
  pub fn new(
    initial_books: HashMap<BookDir, Books>, mut event_rx: broadcast::Receiver<LibraryEvent>,
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
      history_sort: SortConfig::default(),
      bookmarks_sort: SortConfig::default(),
      library_search: "".into(),
      favorites_search: "".into(),
      history_search: "".into(),
      bookmarks_search: "".into(),
      search_tasks: StdHashMap::new(),
    };

    // Initial build
    state.rebuild_and_sort(TargetList::Library);
    state.rebuild_and_sort(TargetList::Favorites);
    state.rebuild_and_sort(TargetList::History);
    state.rebuild_and_sort(TargetList::Bookmarks);

    cx.spawn(|this: gpui::WeakEntity<BooksState>, cx: &mut gpui::AsyncApp| {
      let mut owned_cx = cx.clone();
      async move {
        while let Ok(event) = event_rx.recv().await {
          let _ = this.update(
            &mut owned_cx,
            |state: &mut BooksState, context: &mut gpui::Context<'_, BooksState>| {
              state.apply_event(event);
              context.notify();
            },
          );
        }
      }
    })
    .detach();

    state
  }

  // Universal rebuild method. Executes quickly.
  pub fn rebuild_and_sort(&mut self, target: TargetList) {
    let query = match target {
      TargetList::Library => self.library_search.as_ref(),
      TargetList::Favorites => self.favorites_search.as_ref(),
      TargetList::History => self.history_search.as_ref(),
      TargetList::Bookmarks => self.bookmarks_search.as_ref(),
    }
    .to_lowercase();

    let mut keys = Vec::new();

    for (_, dir_books) in &self.books_map {
      for (_, book) in &dir_books.storage {
        if book.book_path.deleted {
          continue;
        }

        // 1. Check membership for the tab
        let matches_target = match target {
          TargetList::Library => true,
          TargetList::Favorites => book.user_data.favorite,
          TargetList::History => book.user_data.in_history,
          TargetList::Bookmarks => !book.bookmarks.is_empty(),
        };

        if !matches_target {
          continue;
        }

        // 2. Search check
        let matches_search = query.is_empty()
          || book.book_path.name.to_lowercase().contains(&query)
          || book.book_path.parent_dir.dir_name().to_lowercase().contains(&query);

        if matches_search {
          keys.push(book.book_path.clone());
        }
      }
    }

    match target {
      TargetList::Library => self.library_keys = keys,
      TargetList::Favorites => self.favorites_keys = keys,
      TargetList::History => self.history_keys = keys,
      TargetList::Bookmarks => self.bookmarks_keys = keys,
    };

    self.apply_sorting_to(target);
  }

  pub fn set_search_query(&mut self, query: String, target: TargetList, cx: &mut Context<Self>) {
    let current_query = match target {
      TargetList::Library => &mut self.library_search,
      TargetList::Favorites => &mut self.favorites_search,
      TargetList::History => &mut self.history_search,
      TargetList::Bookmarks => &mut self.bookmarks_search,
    };

    if current_query.as_ref() == query {
      return;
    }
    *current_query = query.into();

    let target_capture = target;
    let task = cx.spawn(move |this: WeakEntity<BooksState>, cx: &mut AsyncApp| {
      let mut owned_cx = cx.clone();
      async move {
        owned_cx.background_executor().timer(Duration::from_millis(600)).await;
        let _ = this.update(&mut owned_cx, |state, context| {
          state.rebuild_and_sort(target_capture);
          context.notify();
        });
      }
    });

    self.search_tasks.insert(target, task);
  }

  pub fn get_book(&self, path: &BookPath) -> Option<&Book> {
    let key = path.file_name();
    self.books_map.get(&path.parent_dir).and_then(|dir_books| dir_books.storage.get(&key))
  }

  pub fn set_sort_field(&mut self, field: SortField, target: TargetList, cx: &mut Context<Self>) {
    let config = match target {
      TargetList::Library => &mut self.library_sort,
      TargetList::Favorites => &mut self.favorites_sort,
      TargetList::History => &mut self.history_sort,
      TargetList::Bookmarks => &mut self.bookmarks_sort,
    };
    if config.field != field {
      config.field = field;
      self.apply_sorting_to(target);
      cx.notify();
    }
  }

  pub fn toggle_reverse(&mut self, target: TargetList, cx: &mut Context<Self>) {
    let config = match target {
      TargetList::Library => &mut self.library_sort,
      TargetList::Favorites => &mut self.favorites_sort,
      TargetList::History => &mut self.history_sort,
      TargetList::Bookmarks => &mut self.bookmarks_sort,
    };
    config.is_reversed = !config.is_reversed;
    self.apply_sorting_to(target);
    cx.notify();
  }

  pub fn apply_sorting_to(&mut self, target: TargetList) {
    let (keys, config) = match target {
      TargetList::Library => (&mut self.library_keys, &self.library_sort),
      TargetList::Favorites => (&mut self.favorites_keys, &self.favorites_sort),
      TargetList::History => (&mut self.history_keys, &self.history_sort),
      TargetList::Bookmarks => (&mut self.bookmarks_keys, &self.bookmarks_sort),
    };

    let field = config.field;
    let reversed = config.is_reversed;
    let map = &self.books_map;

    keys.sort_by(|path_a, path_b| {
      let key_a = path_a.file_name();
      let key_b = path_b.file_name();
      let book_a = map.get(&path_a.parent_dir).and_then(|d| d.storage.get(&key_a));
      let book_b = map.get(&path_b.parent_dir).and_then(|d| d.storage.get(&key_b));

      let cmp = match (book_a, book_b) {
        (Some(a), Some(b)) => match field {
          SortField::Name => a.book_path.name.to_lowercase().cmp(&b.book_path.name.to_lowercase()),
          SortField::Size => {
            let BookSize::BYTES(size_a) = a.book_size;
            let BookSize::BYTES(size_b) = b.book_size;
            size_a.cmp(&size_b)
          }
          SortField::Type => a
            .book_path
            .ext
            .to_string()
            .to_lowercase()
            .cmp(&b.book_path.ext.to_string().to_lowercase()),
        },

        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
      };

      if reversed { cmp.reverse() } else { cmp }
    });
  }

  fn apply_event(&mut self, event: LibraryEvent) {
    match event {
      LibraryEvent::BookAdded(book) => {
        let path = book.book_path.clone();
        let dir_books = self.books_map.entry(path.parent_dir.clone()).or_insert_with(|| Books {
          parent_dir: path.parent_dir.clone(),
          storage: HashMap::default(),
        });
        dir_books.storage.insert(path.file_name(), book);
      }

      LibraryEvent::BookUpdated(book) => {
        let path = book.book_path.clone();
        if let Some(dir_books) = self.books_map.get_mut(&path.parent_dir) {
          dir_books.storage.insert(path.file_name(), book);
        }
      }

      LibraryEvent::BookRemoved(path) => {
        if let Some(dir_books) = self.books_map.get_mut(&path.parent_dir) {
          let key = path.file_name();
          dir_books.storage.swap_remove(&key);

          if dir_books.storage.is_empty() {
            self.books_map.swap_remove(&path.parent_dir);
          }
        }
      }

      LibraryEvent::BookPathUpdated { old_path, new_path } => {
        if let Some(dir_books) = self.books_map.get_mut(&old_path.parent_dir)
          && let Some(mut book) = dir_books.storage.swap_remove(&old_path.file_name())
        {
          book.book_path = new_path.clone();
          let new_dir_books =
            self.books_map.entry(new_path.parent_dir.clone()).or_insert_with(|| Books {
              parent_dir: new_path.parent_dir.clone(),
              storage: HashMap::default(),
            });
          new_dir_books.storage.insert(new_path.file_name(), book);
        }
      }

      LibraryEvent::BookMarkAdded { book_path: _ }
      | LibraryEvent::BookMarkUpdated { book_path: _ }
      | LibraryEvent::BookMarkRemoved { book_path: _ } => {}

      LibraryEvent::DirRemoved(dir) => {
        self.books_map.swap_remove(&dir);
      }
    }

    self.rebuild_and_sort(TargetList::Library);
    self.rebuild_and_sort(TargetList::Favorites);
    self.rebuild_and_sort(TargetList::History);
    self.rebuild_and_sort(TargetList::Bookmarks);
  }
}
