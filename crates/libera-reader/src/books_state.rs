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

const DEFAULT_SEARCH_QUERY: &str = "";
const DEBOUNCE_DELAY_MS: u64 = 600;

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
    initial_books: HashMap<BookDir, Books>, event_rx: broadcast::Receiver<LibraryEvent>,
    cx: &mut Context<Self>,
  ) -> Self {
    // Initialize book collections
    let (library_keys, favorites_keys, history_keys, bookmarks_keys) =
      Self::initialize_book_collections(&initial_books);

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
      library_search: DEFAULT_SEARCH_QUERY.into(),
      favorites_search: DEFAULT_SEARCH_QUERY.into(),
      history_search: DEFAULT_SEARCH_QUERY.into(),
      bookmarks_search: DEFAULT_SEARCH_QUERY.into(),
      search_tasks: StdHashMap::new(),
    };

    // Apply initial sorting to all targets
    for target in
      [TargetList::Library, TargetList::Favorites, TargetList::History, TargetList::Bookmarks]
    {
      state.apply_sorting_to(target);
    }

    // Set up event listener for library updates
    // TODO: Fix WeakEntity creation - workaround for now
    // Using a placeholder weak entity - this needs proper implementation
    let weak_entity = gpui::WeakEntity::new_invalid();
    Self::setup_event_listener(weak_entity, event_rx, cx);

    state
  }

  fn initialize_book_collections(
    initial_books: &HashMap<BookDir, Books>,
  ) -> (Vec<BookPath>, Vec<BookPath>, Vec<BookPath>, Vec<BookPath>) {
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

    (library_keys, favorites_keys, history_keys, bookmarks_keys)
  }

  fn setup_event_listener(
    _this: gpui::WeakEntity<BooksState>, mut event_rx: broadcast::Receiver<LibraryEvent>,
    cx: &mut Context<Self>,
  ) {
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
  }

  pub fn rebuild_and_sort(&mut self, target: TargetList) {
    let query = self.get_search_query_for_target(&target).to_lowercase();

    let mut keys = Vec::new();

    for (_, dir_books) in &self.books_map {
      for (_, book) in &dir_books.storage {
        if book.book_path.deleted {
          continue;
        }

        // 1. Check membership for the tab
        let matches_target = self.check_target_membership(book, &target);
        if !matches_target {
          continue;
        }

        // 2. Search check
        let matches_search = self.check_search_match(book, &query);
        if matches_search {
          keys.push(book.book_path.clone());
        }
      }
    }

    self.update_target_keys(target, keys);
    self.apply_sorting_to(target);
  }

  fn get_search_query_for_target(&self, target: &TargetList) -> &str {
    match target {
      TargetList::Library => &self.library_search,
      TargetList::Favorites => &self.favorites_search,
      TargetList::History => &self.history_search,
      TargetList::Bookmarks => &self.bookmarks_search,
    }
    .as_ref()
  }

  fn check_target_membership(&self, book: &Book, target: &TargetList) -> bool {
    match target {
      TargetList::Library => true,
      TargetList::Favorites => book.user_data.favorite,
      TargetList::History => book.user_data.in_history,
      TargetList::Bookmarks => !book.bookmarks.is_empty(),
    }
  }

  fn check_search_match(&self, book: &Book, query: &str) -> bool {
    query.is_empty()
      || book.book_path.name.to_lowercase().contains(query)
      || book.book_path.parent_dir.dir_name().to_lowercase().contains(query)
  }

  fn update_target_keys(&mut self, target: TargetList, keys: Vec<BookPath>) {
    match target {
      TargetList::Library => self.library_keys = keys,
      TargetList::Favorites => self.favorites_keys = keys,
      TargetList::History => self.history_keys = keys,
      TargetList::Bookmarks => self.bookmarks_keys = keys,
    };
  }

  pub fn set_search_query(&mut self, query: String, target: TargetList, cx: &mut Context<Self>) {
    let current_query = self.get_search_query_mut_for_target(target);

    if current_query.as_ref() == query {
      return;
    }
    *current_query = query.into();

    Self::schedule_rebuild_and_sort(target, cx, self);
  }

  fn get_search_query_mut_for_target(&mut self, target: TargetList) -> &mut SharedString {
    match target {
      TargetList::Library => &mut self.library_search,
      TargetList::Favorites => &mut self.favorites_search,
      TargetList::History => &mut self.history_search,
      TargetList::Bookmarks => &mut self.bookmarks_search,
    }
  }

  fn schedule_rebuild_and_sort(target: TargetList, cx: &mut Context<Self>, state: &mut BooksState) {
    let target_capture = target;
    let task = cx.spawn(move |this: WeakEntity<BooksState>, cx: &mut AsyncApp| {
      let mut owned_cx = cx.clone();
      async move {
        owned_cx.background_executor().timer(Duration::from_millis(DEBOUNCE_DELAY_MS)).await;
        let _ = this.update(&mut owned_cx, |state, context| {
          state.rebuild_and_sort(target_capture);
          context.notify();
        });
      }
    });

    state.search_tasks.insert(target, task);
  }

  pub fn get_book(&self, path: &BookPath) -> Option<&Book> {
    let key = path.file_name();
    self.books_map.get(&path.parent_dir).and_then(|dir_books| dir_books.storage.get(&key))
  }

  pub fn set_sort_field(&mut self, field: SortField, target: TargetList, cx: &mut Context<Self>) {
    let config = self.get_sort_config_mut_for_target(target);
    if config.field != field {
      config.field = field;
      self.apply_sorting_to(target);
      cx.notify();
    }
  }

  fn get_sort_config_mut_for_target(&mut self, target: TargetList) -> &mut SortConfig {
    match target {
      TargetList::Library => &mut self.library_sort,
      TargetList::Favorites => &mut self.favorites_sort,
      TargetList::History => &mut self.history_sort,
      TargetList::Bookmarks => &mut self.bookmarks_sort,
    }
  }

  pub fn toggle_reverse(&mut self, target: TargetList, cx: &mut Context<Self>) {
    let config = self.get_sort_config_mut_for_target(target);
    config.is_reversed = !config.is_reversed;
    self.apply_sorting_to(target);
    cx.notify();
  }

  pub fn apply_sorting_to(&mut self, target: TargetList) {
    let books_map_ref = &self.books_map;

    let (keys, config) = match target {
      TargetList::Library => (&mut self.library_keys, &self.library_sort),
      TargetList::Favorites => (&mut self.favorites_keys, &self.favorites_sort),
      TargetList::History => (&mut self.history_keys, &self.history_sort),
      TargetList::Bookmarks => (&mut self.bookmarks_keys, &self.bookmarks_sort),
    };

    let field = config.field;
    let reversed = config.is_reversed;

    keys.sort_by(|path_a, path_b| {
      let key_a = path_a.file_name();
      let key_b = path_b.file_name();
      let book_a = books_map_ref.get(&path_a.parent_dir).and_then(|d| d.storage.get(&key_a));
      let book_b = books_map_ref.get(&path_b.parent_dir).and_then(|d| d.storage.get(&key_b));

      let cmp = Self::compare_books(book_a, book_b, field);

      if reversed { cmp.reverse() } else { cmp }
    });
  }

  fn get_keys_and_config_for_target(
    &mut self, target: TargetList,
  ) -> (&mut Vec<BookPath>, &SortConfig) {
    match target {
      TargetList::Library => (&mut self.library_keys, &self.library_sort),
      TargetList::Favorites => (&mut self.favorites_keys, &self.favorites_sort),
      TargetList::History => (&mut self.history_keys, &self.history_sort),
      TargetList::Bookmarks => (&mut self.bookmarks_keys, &self.bookmarks_sort),
    }
  }

  fn compare_books(book_a: Option<&Book>, book_b: Option<&Book>, field: SortField) -> Ordering {
    match (book_a, book_b) {
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
    }
  }

  fn apply_event(&mut self, event: LibraryEvent) {
    match event {
      LibraryEvent::BookAdded(book) => Self::handle_book_added(self, book),
      LibraryEvent::BookUpdated(book) => Self::handle_book_updated(self, book),
      LibraryEvent::BookRemoved(path) => Self::handle_book_removed(self, path),
      LibraryEvent::BookPathUpdated { old_path, new_path } => {
        Self::handle_book_path_updated(self, old_path, new_path)
      }
      LibraryEvent::BookMarkAdded { book_path }
      | LibraryEvent::BookMarkUpdated { book_path }
      | LibraryEvent::BookMarkRemoved { book_path } => {
        Self::handle_bookmark_updated(self, book_path)
      }
      LibraryEvent::DirRemoved(dir) => Self::handle_directory_removed(self, dir),
    }
  }

  fn handle_book_added(state: &mut BooksState, book: Book) {
    let path = book.book_path.clone();
    let is_favorite = book.user_data.favorite;
    let in_history = book.user_data.in_history;
    let in_bookmark = !book.bookmarks.is_empty();

    let dir_books = state.books_map.entry(path.parent_dir.clone()).or_insert_with(|| Books {
      parent_dir: path.parent_dir.clone(),
      storage: HashMap::default(),
    });
    dir_books.storage.insert(path.file_name(), book);

    state.update_target_collection_if_needed(&path, is_favorite, TargetList::Favorites);
    state.update_target_collection_if_needed(&path, in_history, TargetList::History);
    state.update_target_collection_if_needed(&path, in_bookmark, TargetList::Bookmarks);

    // Always add to library if not deleted and not already present
    if !path.deleted && !state.library_keys.contains(&path) {
      state.library_keys.push(path.clone());
      state.apply_sorting_to(TargetList::Library);
    }
  }

  fn handle_book_updated(state: &mut BooksState, book: Book) {
    let path = book.book_path.clone();
    let is_favorite = book.user_data.favorite;
    let in_history = book.user_data.in_history;
    let is_deleted = book.book_path.deleted;

    // Update in-memory storage
    if let Some(dir_books) = state.books_map.get_mut(&path.parent_dir) {
      dir_books.storage.insert(path.file_name(), book.clone());
    }

    // Handle library updates
    if is_deleted {
      state.library_keys.retain(|k| k != &path);
    } else if !state.library_keys.contains(&path) {
      state.library_keys.push(path.clone());
      state.apply_sorting_to(TargetList::Library);
    }

    // Update favorite status
    state.update_favorite_status(&path, is_favorite);

    // Update history status
    state.update_history_status(&path, in_history);

    // Update bookmark status
    let in_bookmark = !book.bookmarks.is_empty();
    state.update_bookmark_status(&path, in_bookmark);
  }

  fn handle_book_removed(state: &mut BooksState, path: BookPath) {
    if let Some(dir_books) = state.books_map.get_mut(&path.parent_dir) {
      let key = path.file_name();
      dir_books.storage.swap_remove(&key);

      if dir_books.storage.is_empty() {
        state.books_map.swap_remove(&path.parent_dir);
      }
    }

    state.remove_from_all_collections(&path);
  }

  fn handle_book_path_updated(state: &mut BooksState, old_path: BookPath, new_path: BookPath) {
    let mut updated_book = None;
    if let Some(dir_books) = state.books_map.get_mut(&old_path.parent_dir) {
      let old_key = old_path.file_name();
      if let Some(mut book) = dir_books.storage.swap_remove(&old_key) {
        book.book_path = new_path.clone();
        updated_book = Some(book);
      }
    }

    if let Some(book) = updated_book {
      let dir_books = state.books_map.entry(new_path.parent_dir.clone()).or_insert_with(|| Books {
        parent_dir: new_path.parent_dir.clone(),
        storage: HashMap::default(),
      });
      dir_books.storage.insert(new_path.file_name(), book);
    }

    state.update_path_in_all_collections(&old_path, &new_path);

    // Re-sort all collections since paths may have changed
    for target in
      [TargetList::Library, TargetList::Favorites, TargetList::History, TargetList::Bookmarks]
    {
      state.apply_sorting_to(target);
    }
  }

  fn handle_bookmark_updated(state: &mut BooksState, book_path: BookPath) {
    if let Some(book) = state.get_book(&book_path) {
      let in_bookmark = !book.bookmarks.is_empty();
      state.update_bookmark_status(&book_path, in_bookmark);
    }
  }

  fn handle_directory_removed(state: &mut BooksState, dir: BookDir) {
    state.books_map.swap_remove(&dir);
    state.remove_paths_with_parent_dir(&dir);
  }

  fn update_target_collection_if_needed(
    &mut self, path: &BookPath, condition: bool, target: TargetList,
  ) {
    if condition && !self.keys_for_target(target).contains(path) {
      self.keys_for_target_mut(target).push(path.clone());
      self.apply_sorting_to(target);
    }
  }

  fn update_favorite_status(&mut self, path: &BookPath, is_favorite: bool) {
    if is_favorite && !self.favorites_keys.contains(path) {
      self.favorites_keys.push(path.clone());
      self.apply_sorting_to(TargetList::Favorites);
    } else if !is_favorite {
      self.favorites_keys.retain(|k| k != path);
    }
  }

  fn update_history_status(&mut self, path: &BookPath, in_history: bool) {
    if in_history && !self.history_keys.contains(path) {
      self.history_keys.push(path.clone());
      self.apply_sorting_to(TargetList::History);
    } else if !in_history {
      self.history_keys.retain(|k| k != path);
    }
  }

  fn update_bookmark_status(&mut self, path: &BookPath, in_bookmark: bool) {
    if in_bookmark && !self.bookmarks_keys.contains(path) {
      self.bookmarks_keys.push(path.clone());
      self.apply_sorting_to(TargetList::Bookmarks);
    } else if !in_bookmark {
      self.bookmarks_keys.retain(|k| k != path);
    }
  }

  fn update_path_in_all_collections(&mut self, old_path: &BookPath, new_path: &BookPath) {
    for target in
      [TargetList::Library, TargetList::Favorites, TargetList::History, TargetList::Bookmarks]
    {
      self.replace_path_in_collection(target, old_path, new_path);
    }
  }

  fn remove_from_all_collections(&mut self, path: &BookPath) {
    for target in
      [TargetList::Library, TargetList::Favorites, TargetList::History, TargetList::Bookmarks]
    {
      self.keys_for_target_mut(target).retain(|k| k != path);
    }
  }

  fn remove_paths_with_parent_dir(&mut self, dir: &BookDir) {
    for target in
      [TargetList::Library, TargetList::Favorites, TargetList::History, TargetList::Bookmarks]
    {
      self.keys_for_target_mut(target).retain(|k| k.parent_dir != *dir);
    }
  }

  fn replace_path_in_collection(
    &mut self, target: TargetList, old_path: &BookPath, new_path: &BookPath,
  ) {
    let keys = self.keys_for_target_mut(target);
    if let Some(pos) = keys.iter().position(|k| k == old_path) {
      keys[pos] = new_path.clone();
    }
  }

  fn keys_for_target(&self, target: TargetList) -> &Vec<BookPath> {
    match target {
      TargetList::Library => &self.library_keys,
      TargetList::Favorites => &self.favorites_keys,
      TargetList::History => &self.history_keys,
      TargetList::Bookmarks => &self.bookmarks_keys,
    }
  }

  fn keys_for_target_mut(&mut self, target: TargetList) -> &mut Vec<BookPath> {
    match target {
      TargetList::Library => &mut self.library_keys,
      TargetList::Favorites => &mut self.favorites_keys,
      TargetList::History => &mut self.history_keys,
      TargetList::Bookmarks => &mut self.bookmarks_keys,
    }
  }
}
