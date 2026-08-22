use crate::books_state::{BooksState, TargetList};
use crate::ctx::Ctx;
use crate::db::models::books::book::BookPath;
use gpui::{App, BorrowAppContext, Entity};
use std::time::{SystemTime, UNIX_EPOCH};

/// Record that a book was opened: bump its `last_opened`, ensure it is in the
/// history list, and persist the change to the database.
pub fn push_at_history(path: BookPath, books_state: Entity<BooksState>, cx: &mut App) {
  let id: gpui::SharedString = path.full_path_string();
  let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
  let mut should_persist = false;

  books_state.update(cx, |state, cx| {
    if let Some(light) = state.books_map.get_mut(&id) {
      light.last_opened = now;
      should_persist = true;

      if !state.history_keys.contains(&id) {
        state.history_keys.push(id.clone());
      }
      state.rebuild_and_sort(TargetList::History);
      cx.notify();
    }
  });

  if should_persist {
    cx.update_global::<Ctx, _>(|ctx, _cx| {
      if let Ok(Some(book)) = ctx.db.get_book(path.clone()) {
        let mut updated = book.clone();
        updated.user_data.last_opened = now;
        let _ = ctx.db.update_book(updated);
      }
    });
  }
}

/// Toggle the favorite flag on a book, rebuild the favorites list, and persist.
pub fn toggle_favorite(path: BookPath, books_state: Entity<BooksState>, cx: &mut App) {
  let id: gpui::SharedString = path.full_path_string();
  let mut new_state = None;

  books_state.update(cx, |state, cx| {
    if let Some(light) = state.books_map.get_mut(&id) {
      light.is_favorite = !light.is_favorite;
      new_state = Some(light.is_favorite);
      // Favorites list is rebuilt from `is_favorite` flags by `rebuild_and_sort`.
      state.rebuild_and_sort(TargetList::Favorites);
      cx.notify();
    }
  });

  if let Some(is_favorite) = new_state {
    cx.update_global::<Ctx, _>(|ctx, _cx| {
      if let Ok(Some(book)) = ctx.db.get_book(path.clone()) {
        let mut updated = book.clone();
        updated.user_data.favorite = is_favorite;
        let _ = ctx.db.update_book(updated);
      }
    });
  }
}
