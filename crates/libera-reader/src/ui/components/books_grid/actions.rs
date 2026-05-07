use crate::books_state::{BooksState, TargetList};
use gpui::{App, BorrowAppContext, Entity};
use libera_reader_core::ctx::Ctx;
use libera_reader_core::db::models::books::book::BookPath;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn push_at_history(path: BookPath, books_state: Entity<BooksState>, cx: &mut App) {
  let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
  let mut should_persist = false;
  books_state.update(cx, |state, cx| {
    if let Some(dir_books) = state.books_map.get_mut(&path.parent_dir) {
      let key = path.file_name();
      if let Some(book) = dir_books.storage.get_mut(&key) {
        book.user_data.last_opened = now;
        should_persist = true;

        if !state.history_keys.contains(&path) {
          state.history_keys.push(path.clone());
        }

        state.apply_sorting_to(TargetList::History);
        cx.notify();
      }
    }
  });

  if should_persist {
    cx.update_global(|ctx: &mut Ctx, _cx| {
      if let Ok(Some(book)) = ctx.db.get_book(path.clone()) {
        let mut updated_book = book.clone();
        updated_book.user_data.last_opened = now;
        let _ = ctx.db.update_book(updated_book);
      }
    });
  }
}

pub fn toggle_favorite(path: BookPath, books_state: Entity<BooksState>, cx: &mut App) {
  let mut next_favorite_state = None;
  books_state.update(cx, |state, cx| {
    if let Some(dir_books) = state.books_map.get_mut(&path.parent_dir) {
      let key = path.file_name();
      if let Some(book) = dir_books.storage.get_mut(&key) {
        book.user_data.favorite = !book.user_data.favorite;
        next_favorite_state = Some(book.user_data.favorite);
        if book.user_data.favorite {
          if !state.favorites_keys.contains(&path) {
            state.favorites_keys.push(path.clone());
            state.apply_sorting_to(TargetList::Favorites);
          }
        } else {
          state.favorites_keys.retain(|k| k != &path);
        }
        cx.notify();
      }
    }
  });

  if let Some(is_favorite) = next_favorite_state {
    cx.update_global(|ctx: &mut Ctx, _cx| {
      if let Ok(Some(book)) = ctx.db.get_book(path.clone()) {
        let mut updated_book = book.clone();
        updated_book.user_data.favorite = is_favorite;
        let _ = ctx.db.update_book(updated_book);
      }
    });
  }
}
