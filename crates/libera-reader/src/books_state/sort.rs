use super::{BooksState, BooksStateData, SortConfig, SortField, TargetList};
use gpui::Context;
use std::cmp::Ordering;

impl BooksStateData {
  pub fn sort_config(&self, target: TargetList) -> &SortConfig {
    match target {
      TargetList::Library => &self.library_sort,
      TargetList::Favorites => &self.favorites_sort,
      TargetList::History => &self.history_sort,
      TargetList::Bookmarks => &self.bookmarks_sort,
    }
  }

  pub fn sort_config_mut(&mut self, target: TargetList) -> &mut SortConfig {
    match target {
      TargetList::Library => &mut self.library_sort,
      TargetList::Favorites => &mut self.favorites_sort,
      TargetList::History => &mut self.history_sort,
      TargetList::Bookmarks => &mut self.bookmarks_sort,
    }
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

    keys.sort_by(|id_a, id_b| {
      let book_a = map.get(id_a);
      let book_b = map.get(id_b);

      let cmp = match (book_a, book_b) {
        (Some(a), Some(b)) => match field {
          SortField::Name => a.book_path.name.to_lowercase().cmp(&b.book_path.name.to_lowercase()),
          SortField::Size => a.size_bytes().cmp(&b.size_bytes()),
          SortField::Type => a
            .book_path
            .ext
            .to_string()
            .to_lowercase()
            .cmp(&b.book_path.ext.to_string().to_lowercase()),
          SortField::LastOpened => a.user_data.last_opened.cmp(&b.user_data.last_opened),
        },
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
      };

      if reversed { cmp.reverse() } else { cmp }
    });
  }
}

impl BooksState {
  #[allow(dead_code)]
  pub fn set_sort_field(&self, field: SortField, target: TargetList, cx: &mut Context<Self>) {
    let mut data = self.write();
    if data.sort_config(target).field != field {
      data.sort_config_mut(target).field = field;
      data.apply_sorting_to(target);
      drop(data);
      cx.notify();
    }
  }

  #[allow(dead_code)]
  pub fn toggle_reverse(&self, target: TargetList, cx: &mut Context<Self>) {
    let mut data = self.write();
    data.sort_config_mut(target).is_reversed = !data.sort_config(target).is_reversed;
    data.apply_sorting_to(target);
    drop(data);
    cx.notify();
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::books_state::BooksMap;
  use crate::books_state::thumbnails::ThumbnailCache;
  use crate::db::models::UserData;
  use crate::db::models::books::book::{Book, BookPath, BookSize};
  use crate::types::HashMap;
  use std::path::PathBuf;

  fn make_dummy_book(name: &str, ext_str: &str, size: u64, last_opened: u64) -> Book {
    let path_str = format!("{}/{}.{}", "/dummy", name, ext_str);
    let mut book_path = BookPath::new(std::path::Path::new(&path_str)).unwrap();
    book_path.name = name.into();
    let parent_dir = book_path.parent_dir.clone();
    Book {
      parent_dir,
      book_path,
      book_size: BookSize::BYTES(size),
      user_data: UserData { favorite: false, last_opened },
      bookmark_count: 0,
    }
  }

  #[test]
  fn test_sorting_by_name_and_reverse() {
    let mut books_map = BooksMap::default();
    let b1 = make_dummy_book("Beta", "pdf", 100, 10);
    let b2 = make_dummy_book("Alpha", "epub", 200, 20);
    let b3 = make_dummy_book("Gamma", "pdf", 50, 5);

    books_map.insert("1".into(), b1);
    books_map.insert("2".into(), b2);
    books_map.insert("3".into(), b3);

    let mut state = BooksStateData {
      books_map,
      thumbnails: ThumbnailCache::new(PathBuf::from("/tmp")),
      library_keys: vec!["1".into(), "2".into(), "3".into()],
      favorites_keys: vec![],
      history_keys: vec![],
      bookmarks_keys: vec![],
      library_sort: SortConfig { field: SortField::Name, is_reversed: false },
      favorites_sort: SortConfig::default(),
      history_sort: SortConfig::default(),
      bookmarks_sort: SortConfig::default(),
      library_search: "".into(),
      favorites_search: "".into(),
      history_search: "".into(),
      bookmarks_search: "".into(),
      search_generation: HashMap::default(),
    };

    state.apply_sorting_to(TargetList::Library);
    let keys: Vec<&str> = state.library_keys.iter().map(|s| s.as_ref()).collect();
    assert_eq!(keys, vec!["2", "1", "3"]); // Alpha, Beta, Gamma

    state.library_sort.is_reversed = true;
    state.apply_sorting_to(TargetList::Library);
    let rev_keys: Vec<&str> = state.library_keys.iter().map(|s| s.as_ref()).collect();
    assert_eq!(rev_keys, vec!["3", "1", "2"]); // Gamma, Beta, Alpha
  }

  #[test]
  fn test_sorting_by_size() {
    let mut books_map = BooksMap::default();
    let b1 = make_dummy_book("B", "pdf", 300, 10);
    let b2 = make_dummy_book("A", "epub", 100, 20);
    let b3 = make_dummy_book("C", "pdf", 200, 5);

    books_map.insert("1".into(), b1);
    books_map.insert("2".into(), b2);
    books_map.insert("3".into(), b3);

    let mut state = BooksStateData {
      books_map,
      thumbnails: ThumbnailCache::new(PathBuf::from("/tmp")),
      library_keys: vec!["1".into(), "2".into(), "3".into()],
      favorites_keys: vec![],
      history_keys: vec![],
      bookmarks_keys: vec![],
      library_sort: SortConfig { field: SortField::Size, is_reversed: false },
      favorites_sort: SortConfig::default(),
      history_sort: SortConfig::default(),
      bookmarks_sort: SortConfig::default(),
      library_search: "".into(),
      favorites_search: "".into(),
      history_search: "".into(),
      bookmarks_search: "".into(),
      search_generation: HashMap::default(),
    };

    state.apply_sorting_to(TargetList::Library);
    let keys: Vec<&str> = state.library_keys.iter().map(|s| s.as_ref()).collect();
    assert_eq!(keys, vec!["2", "3", "1"]); // 100, 200, 300
  }
}
