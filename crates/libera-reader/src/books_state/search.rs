use super::{BooksState, BooksStateData, TargetList};
use gpui::{AsyncApp, Context, WeakEntity};
use std::time::Duration;

#[allow(dead_code)]
const DEBOUNCE_DELAY_MS: u64 = 600;

impl BooksStateData {
  pub fn rebuild_and_sort(&mut self, target: TargetList) {
    let query = match target {
      TargetList::Library => self.library_search.as_ref(),
      TargetList::Favorites => self.favorites_search.as_ref(),
      TargetList::History => self.history_search.as_ref(),
      TargetList::Bookmarks => self.bookmarks_search.as_ref(),
    }
    .to_lowercase();

    let mut keys = Vec::new();

    for (id, book) in &self.books_map {
      if book.book_path.deleted {
        continue;
      }

      let matches_target = match target {
        TargetList::Library => true,
        TargetList::Favorites => book.user_data.favorite,
        TargetList::History => book.user_data.last_opened > 0,
        TargetList::Bookmarks => book.bookmark_count > 0,
      };

      if !matches_target {
        continue;
      }

      let matches_search = query.is_empty()
        || book.book_path.name.to_lowercase().contains(&query)
        || book.parent_dir.to_lowercase().contains(&query);
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

impl BooksState {
  #[allow(dead_code)]
  pub fn set_search_query(&self, query: String, target: TargetList, cx: &mut Context<Self>) {
    let mut data = self.write();
    let current_query = match target {
      TargetList::Library => &mut data.library_search,
      TargetList::Favorites => &mut data.favorites_search,
      TargetList::History => &mut data.history_search,
      TargetList::Bookmarks => &mut data.bookmarks_search,
    };

    if current_query.as_ref() == query {
      return;
    }
    *current_query = query.into();

    // Increment generation counter to invalidate previous pending tasks
    let generation = data.search_generation.entry(target).or_insert(0);
    *generation += 1;
    let my_gen = *generation;
    drop(data);

    cx.spawn(move |this: WeakEntity<BooksState>, cx: &mut AsyncApp| {
      let mut owned_cx = cx.clone();
      async move {
        owned_cx.background_executor().timer(Duration::from_millis(DEBOUNCE_DELAY_MS)).await;
        let _ = this.update(&mut owned_cx, |state, context| {
          let mut data = state.write();
          // Skip if a newer search was already issued
          if data.search_generation.get(&target) != Some(&my_gen) {
            return;
          }
          data.rebuild_and_sort(target);
          drop(data);
          context.notify();
        });
      }
    })
    .detach();
  }
}
