use crate::app_ext::AppExt;
use crate::books_state::TargetList;
use crate::db::models::books::book::BookPath;
use gpui::App;
use std::time::{SystemTime, UNIX_EPOCH};

/// Entry point when a user clicks on a book card in any view mode.
/// Records the book in history and will trigger reading / viewer navigation.
pub fn book_card_click(path: BookPath, cx: &mut App) {
  push_at_history(path, cx);
}

/// Record that a book was opened: bump its `last_opened`, ensure it is in the
/// history list, and persist the change to the database.
pub fn push_at_history(path: BookPath, cx: &mut App) {
  let id: gpui::SharedString = path.full_path_string();
  let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
  let mut should_persist = false;
  let books_state = cx.books_state_entity().clone();

  books_state.update(cx, |state, cx| {
    let mut data = state.write();
    if let Some(book) = data.books_map.get_mut(&id) {
      book.user_data.last_opened = now;
      should_persist = true;

      if !data.history_keys.contains(&id) {
        data.history_keys.push(id.clone());
      }
      data.rebuild_and_sort(TargetList::History);
      drop(data);
      cx.notify();
    }
  });

  if should_persist {
    let db = cx.db().clone();
    if let Ok(Some(book)) = crate::db::models::books::book::Book::get(&db, path.clone()) {
      let mut updated = book.clone();
      updated.user_data.last_opened = now;
      let _ = crate::db::models::books::book::Book::update(&db, updated);
    }
  }
}

/// Toggle the favorite flag on a book, rebuild the favorites list, and persist.
pub fn toggle_favorite(path: BookPath, cx: &mut App) {
  let id: gpui::SharedString = path.full_path_string();
  let mut new_state = None;
  let books_state = cx.books_state_entity().clone();

  books_state.update(cx, |state, cx| {
    let mut data = state.write();
    if let Some(book) = data.books_map.get_mut(&id) {
      book.user_data.favorite = !book.user_data.favorite;
      new_state = Some(book.user_data.favorite);
      data.rebuild_and_sort(TargetList::Favorites);
      drop(data);
      cx.notify();
    }
  });

  if let Some(is_favorite) = new_state {
    let db = cx.db().clone();
    if let Ok(Some(book)) = crate::db::models::books::book::Book::get(&db, path.clone()) {
      let mut updated = book.clone();
      updated.user_data.favorite = is_favorite;
      let _ = crate::db::models::books::book::Book::update(&db, updated);
    }
  }
}
