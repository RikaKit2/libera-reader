use super::{BooksState, TargetList};
use gpui::{AsyncApp, Context, WeakEntity};
use std::time::Duration;

const DEBOUNCE_DELAY_MS: u64 = 600;

impl BooksState {
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

    let task = cx.spawn(move |this: WeakEntity<BooksState>, cx: &mut AsyncApp| {
      let mut owned_cx = cx.clone();
      async move {
        owned_cx.background_executor().timer(Duration::from_millis(DEBOUNCE_DELAY_MS)).await;
        let _ = this.update(&mut owned_cx, |state, context| {
          state.rebuild_and_sort(target);
          context.notify();
        });
      }
    });

    self.search_tasks.insert(target, task);
  }

  pub fn rebuild_and_sort(&mut self, target: TargetList) {
    let query = match target {
      TargetList::Library => self.library_search.as_ref(),
      TargetList::Favorites => self.favorites_search.as_ref(),
      TargetList::History => self.history_search.as_ref(),
      TargetList::Bookmarks => self.bookmarks_search.as_ref(),
    }
    .to_lowercase();

    let mut keys = Vec::new();

    for (id, light) in &self.books_map {
      if light.deleted {
        continue;
      }

      let matches_target = match target {
        TargetList::Library => true,
        TargetList::Favorites => light.is_favorite,
        TargetList::History => light.last_opened > 0,
        TargetList::Bookmarks => {
          // Bookmarks list is built from the DB on request; we skip here
          // since we don't store bookmarks in LightBook.
          // For now, just skip bookmarks in memory.
          false
        }
      };

      if !matches_target {
        continue;
      }

      let matches_search = query.is_empty()
        || light.name.to_lowercase().contains(&query)
        || light.parent_dir.to_lowercase().contains(&query);

      if matches_search {
        keys.push(id.clone());
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
}
