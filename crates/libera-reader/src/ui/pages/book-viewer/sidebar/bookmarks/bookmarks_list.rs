use crate::app_ext::AppExt;
use crate::db::models::books::bookmark::{BookBookmarks, BookMark};
use crate::ui::pages::book_viewer::sidebar::bookmarks::bookmark_item::BookmarkItem;
use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::ActiveTheme;

pub struct BookmarksList {
  state: Entity<BookViewerState>,
}

impl BookmarksList {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl Render for BookmarksList {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let current_book = self.state.read(cx).current_book.clone();
    let db = cx.db().clone();

    let bookmarks: Vec<BookMark> = if let Some(path) = current_book {
      BookBookmarks::get(&db, path).unwrap_or_default()
    } else {
      Vec::new()
    };

    if bookmarks.is_empty() {
      return div()
        .w_full()
        .py_8()
        .flex()
        .items_center()
        .justify_center()
        .text_sm()
        .text_color(cx.theme().muted_foreground)
        .child("Нет закладок для этой книги");
    }

    let state = self.state.clone();

    div()
      .w_full()
      .flex()
      .flex_col()
      .gap_y_1_5()
      .children(bookmarks.into_iter().map(|bm| cx.new(|_cx| BookmarkItem::new(bm, state.clone()))))
  }
}
