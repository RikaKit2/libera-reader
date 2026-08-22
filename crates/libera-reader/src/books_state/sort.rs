use super::{BooksState, SortField, TargetList};
use gpui::Context;
use std::cmp::Ordering;

impl BooksState {
  #[allow(dead_code)]
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

  #[allow(dead_code)]
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

    keys.sort_by(|id_a, id_b| {
      let book_a = map.get(id_a);
      let book_b = map.get(id_b);

      let cmp = match (book_a, book_b) {
        (Some(a), Some(b)) => match field {
          SortField::Name => a.name_lower.cmp(&b.name_lower),
          SortField::Size => a.size.cmp(&b.size),
          SortField::Type => a.ext_lower.cmp(&b.ext_lower),
          SortField::LastOpened => a.last_opened.cmp(&b.last_opened),
        },
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
      };

      if reversed { cmp.reverse() } else { cmp }
    });
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::books_state::models::{LightBook, SortConfig};
  use crate::books_state::thumbnails::ThumbnailCache;
  use std::collections::HashMap as StdHashMap;
  use std::path::PathBuf;

  fn make_dummy_light_book(
    id: &str, name: &str, ext: &str, size: u64, last_opened: u64,
  ) -> LightBook {
    LightBook {
      id: id.into(),
      name: name.into(),
      name_lower: name.to_lowercase().into(),
      ext: ext.into(),
      ext_lower: ext.to_lowercase().into(),
      ext_upper: ext.to_uppercase().into(),
      size,
      size_label: format!("{} MB", size).into(),
      format_label: ext.into(),
      cover_btn_id: format!("cover_{}", id).into(),
      fav_btn_id: format!("fav_{}", id).into(),
      last_opened,
      is_favorite: false,
      has_thumbnail: false,
      formatted_title: name.into(),
      parent_dir: "/dummy".into(),
      deleted: false,
      bookmark_count: 0,
    }
  }

  #[test]
  fn test_sorting_by_name_and_reverse() {
    let mut books_map = StdHashMap::new();
    let b1 = make_dummy_light_book("1", "Beta", "pdf", 100, 10);
    let b2 = make_dummy_light_book("2", "Alpha", "epub", 200, 20);
    let b3 = make_dummy_light_book("3", "Gamma", "pdf", 50, 5);

    books_map.insert(b1.id.clone(), b1);
    books_map.insert(b2.id.clone(), b2);
    books_map.insert(b3.id.clone(), b3);

    let mut state = BooksState {
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
      search_tasks: StdHashMap::new(),
      search_generation: StdHashMap::new(),
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
    let mut books_map = StdHashMap::new();
    let b1 = make_dummy_light_book("1", "B", "pdf", 300, 10);
    let b2 = make_dummy_light_book("2", "A", "epub", 100, 20);
    let b3 = make_dummy_light_book("3", "C", "pdf", 200, 5);

    books_map.insert(b1.id.clone(), b1);
    books_map.insert(b2.id.clone(), b2);
    books_map.insert(b3.id.clone(), b3);

    let mut state = BooksState {
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
      search_tasks: StdHashMap::new(),
      search_generation: StdHashMap::new(),
    };

    state.apply_sorting_to(TargetList::Library);
    let keys: Vec<&str> = state.library_keys.iter().map(|s| s.as_ref()).collect();
    assert_eq!(keys, vec!["2", "3", "1"]); // 100, 200, 300
  }
}
