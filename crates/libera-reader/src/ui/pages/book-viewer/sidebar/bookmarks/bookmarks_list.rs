use crate::app_ext::AppExt;
use crate::db::models::books::bookmark::{BookBookmarks, BookMark};
use crate::ui::pages::book_viewer::constants::bookmarks;
use crate::ui::pages::book_viewer::sidebar::bookmarks::bookmark_item::BookmarkItem;
use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::ActiveTheme;
use gpui_component::StyledExt;
use rust_i18n::t;

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
    let theme = cx.theme();
    let current_book = self.state.read(cx).current_book.clone();
    let query = self.state.read(cx).bookmark_search_query.trim().to_lowercase();
    let db = cx.db().clone();

    let all_bookmarks: Vec<BookMark> = if let Some(path) = current_book {
      BookBookmarks::get(&db, path).unwrap_or_default()
    } else {
      Vec::new()
    };

    if all_bookmarks.is_empty() {
      return div()
        .w_full()
        .py(bookmarks::EMPTY_PY)
        .px(bookmarks::EMPTY_PX)
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_y(bookmarks::EMPTY_GAP_Y)
        .child(
          svg()
            .path("heroicons--bookmark-20-solid.svg")
            .size(bookmarks::EMPTY_ICON_SIZE)
            .text_color(theme.muted_foreground.opacity(0.4)),
        )
        .child(
          div()
            .text_sm()
            .font_medium()
            .text_color(theme.muted_foreground)
            .child(t!("components.book_viewer.bookmarks.empty_title").to_string()),
        )
        .child(
          div()
            .text_xs()
            .text_color(theme.muted_foreground.opacity(0.7))
            .text_center()
            .child(t!("components.book_viewer.bookmarks.empty_description").to_string()),
        );
    }

    let filtered_bookmarks: Vec<BookMark> = if query.is_empty() {
      all_bookmarks
    } else {
      all_bookmarks
        .into_iter()
        .filter(|bm| {
          bm.title.to_lowercase().contains(&query)
            || bm.content.to_lowercase().contains(&query)
            || bm.page_number.to_string().contains(&query)
        })
        .collect()
    };

    if filtered_bookmarks.is_empty() {
      return div()
        .w_full()
        .py(bookmarks::EMPTY_PY)
        .px(bookmarks::EMPTY_PX)
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_y(bookmarks::EMPTY_GAP_Y)
        .child(
          svg()
            .path("heroicons--magnifying-glass.svg")
            .size(bookmarks::EMPTY_ICON_SIZE)
            .text_color(theme.muted_foreground.opacity(0.4)),
        )
        .child(
          div()
            .text_sm()
            .font_medium()
            .text_color(theme.muted_foreground)
            .child(t!("components.book_viewer.bookmarks.not_found_title").to_string()),
        )
        .child(
          div()
            .text_xs()
            .text_color(theme.muted_foreground.opacity(0.7))
            .text_center()
            .child(t!("components.book_viewer.bookmarks.not_found_description").to_string()),
        );
    }

    let state = self.state.clone();

    div().w_full().flex().flex_col().gap_y(bookmarks::LIST_GAP_Y).children(
      filtered_bookmarks.into_iter().map(|bm| cx.new(|_cx| BookmarkItem::new(bm, state.clone()))),
    )
  }
}
