#![allow(dead_code)]
#![allow(unused_imports)]

use gpui::*;
use gpui_component::select::SelectItem;
use rust_i18n::t;
use std::cmp::Ordering;
use std::fmt;
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

#[derive(Clone, Copy)]
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
}

impl BooksState {
  pub fn new(
    initial_books: HashMap<BookDir, Books>, mut event_rx: broadcast::Receiver<LibraryEvent>,
    cx: &mut Context<Self>,
  ) -> Self {
    let mut library_keys = Vec::new();
    let mut favorites_keys = Vec::new();
    let mut history_keys = Vec::new();
    let mut bookmarks_keys = Vec::new();

    for (_, dir_books) in initial_books.iter() {
      for (_, book) in dir_books.storage.iter() {
        if !book.book_path.deleted {
          library_keys.push(book.book_path.clone());
        }
        if book.user_data.favorite {
          favorites_keys.push(book.book_path.clone());
        }
        if book.user_data.in_history {
          history_keys.push(book.book_path.clone());
        }
        if !book.bookmarks.is_empty() {
          bookmarks_keys.push(book.book_path.clone());
        }
      }
    }

    let mut state = Self {
      books_map: initial_books,
      library_keys,
      favorites_keys,
      history_keys,
      bookmarks_keys,
      library_sort: SortConfig::default(),
      favorites_sort: SortConfig::default(),
      history_sort: SortConfig::default(),
      bookmarks_sort: SortConfig::default(),
    };

    state.apply_sorting_to(TargetList::Library);
    state.apply_sorting_to(TargetList::Favorites);
    state.apply_sorting_to(TargetList::History);
    state.apply_sorting_to(TargetList::Bookmarks);

    cx.spawn(|this: gpui::WeakEntity<BooksState>, cx: &mut gpui::AsyncApp| {
      let mut owned_cx = cx.clone();
      async move {
        while let Ok(event) = event_rx.recv().await {
          let _ =
            this.update(&mut owned_cx, |state: &mut BooksState, context: &mut gpui::Context<'_, BooksState>| {
              state.apply_event(event);
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
          SortField::Type => {
            a.book_path.ext.to_string().to_lowercase().cmp(&b.book_path.ext.to_string().to_lowercase())
          }
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
        let is_favorite = book.user_data.favorite;
        let in_history = book.user_data.in_history;
        let in_bookmark = !book.bookmarks.is_empty();

        let dir_books = self
          .books_map
          .entry(path.parent_dir.clone())
          .or_insert_with(|| Books { parent_dir: path.parent_dir.clone(), storage: HashMap::default() });
        dir_books.storage.insert(path.file_name(), book);

        if !self.library_keys.contains(&path) {
          self.library_keys.push(path.clone());
          self.apply_sorting_to(TargetList::Library);
        }

        if is_favorite && !self.favorites_keys.contains(&path) {
          self.favorites_keys.push(path.clone());
          self.apply_sorting_to(TargetList::Favorites);
        }

        if in_history && !self.history_keys.contains(&path) {
          self.history_keys.push(path.clone());
          self.apply_sorting_to(TargetList::History);
        }

        if in_bookmark && !self.bookmarks_keys.contains(&path) {
          self.bookmarks_keys.push(path.clone());
          self.apply_sorting_to(TargetList::Bookmarks);
        }
      }

      LibraryEvent::BookUpdated(book) => {
        let path = book.book_path.clone();
        let is_favorite = book.user_data.favorite;
        let in_history = book.user_data.in_history;
        let is_deleted = book.book_path.deleted;

        if let Some(dir_books) = self.books_map.get_mut(&path.parent_dir) {
          dir_books.storage.insert(path.file_name(), book.clone());
        }

        if is_deleted {
          self.library_keys.retain(|k| k != &path);
        } else if !self.library_keys.contains(&path) {
          self.library_keys.push(path.clone());
          self.apply_sorting_to(TargetList::Library);
        }

        if is_favorite && !self.favorites_keys.contains(&path) {
          self.favorites_keys.push(path.clone());
          self.apply_sorting_to(TargetList::Favorites);
        } else if !is_favorite {
          self.favorites_keys.retain(|k| k != &path);
        }

        if in_history && !self.history_keys.contains(&path) {
          self.history_keys.push(path.clone());
          self.apply_sorting_to(TargetList::History);
        } else if !in_history {
          self.history_keys.retain(|k| k != &path);
        }

        let in_bookmark = !book.bookmarks.is_empty();
        if in_bookmark && !self.bookmarks_keys.contains(&path) {
          self.bookmarks_keys.push(path.clone());
          self.apply_sorting_to(TargetList::Bookmarks);
        } else if !in_bookmark {
          self.bookmarks_keys.retain(|k| k != &path);
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

        self.library_keys.retain(|k| k != &path);
        self.favorites_keys.retain(|k| k != &path);
        self.history_keys.retain(|k| k != &path);
        self.bookmarks_keys.retain(|k| k != &path);
      }

      LibraryEvent::BookPathUpdated { old_path, new_path } => {
        let mut updated_book = None;
        if let Some(dir_books) = self.books_map.get_mut(&old_path.parent_dir) {
          let old_key = old_path.file_name();
          if let Some(mut book) = dir_books.storage.swap_remove(&old_key) {
            book.book_path = new_path.clone();
            updated_book = Some(book);
          }
        }

        if let Some(book) = updated_book {
          let dir_books = self
            .books_map
            .entry(new_path.parent_dir.clone())
            .or_insert_with(|| Books { parent_dir: new_path.parent_dir.clone(), storage: HashMap::default() });
          dir_books.storage.insert(new_path.file_name(), book);
        }

        for key in &mut self.library_keys {
          if *key == old_path {
            *key = new_path.clone();
          }
        }
        for key in &mut self.favorites_keys {
          if *key == old_path {
            *key = new_path.clone();
          }
        }
        for key in &mut self.history_keys {
          if *key == old_path {
            *key = new_path.clone();
          }
        }
        for key in &mut self.bookmarks_keys {
          if *key == old_path {
            *key = new_path.clone();
          }
        }

        self.apply_sorting_to(TargetList::Library);
        self.apply_sorting_to(TargetList::Favorites);
        self.apply_sorting_to(TargetList::History);
        self.apply_sorting_to(TargetList::Bookmarks);
      }

      LibraryEvent::BookMarkAdded { book_path }
      | LibraryEvent::BookMarkUpdated { book_path }
      | LibraryEvent::BookMarkRemoved { book_path } => {
        if let Some(book) = self.get_book(&book_path) {
          let in_bookmark = !book.bookmarks.is_empty();
          if in_bookmark && !self.bookmarks_keys.contains(&book_path) {
            self.bookmarks_keys.push(book_path.clone());
            self.apply_sorting_to(TargetList::Bookmarks);
          } else if !in_bookmark {
            self.bookmarks_keys.retain(|k| k != &book_path);
          }
        }
      }

      LibraryEvent::DirRemoved(dir) => {
        self.books_map.swap_remove(&dir);
        self.library_keys.retain(|k| k.parent_dir != dir);
        self.favorites_keys.retain(|k| k.parent_dir != dir);
        self.history_keys.retain(|k| k.parent_dir != dir);
        self.bookmarks_keys.retain(|k| k.parent_dir != dir);
      }
    }
  }
}
