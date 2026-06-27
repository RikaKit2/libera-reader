use super::{BooksState, SortField, TargetList};
use gpui::Context;
use std::cmp::Ordering;

impl BooksState {
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

    keys.sort_by(|id_a, id_b| {
      let book_a = map.get(id_a);
      let book_b = map.get(id_b);

      let cmp = match (book_a, book_b) {
        (Some(a), Some(b)) => match field {
          SortField::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
          SortField::Size => a.size.cmp(&b.size),
          SortField::Type => a.ext.to_lowercase().cmp(&b.ext.to_lowercase()),
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
