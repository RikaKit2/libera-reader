use crate::app_ext::AppExt;
use crate::db::models::books::bookmark::{BookBookmarks, BookMark};
use crate::ui::pages::book_viewer::constants::{RADIUS_MD, bookmarks};
use crate::ui::pages::book_viewer::state::BookViewerState;
use chrono::Local;
use gpui::*;
use gpui_component::ActiveTheme;
use gpui_component::StyledExt;
use rust_i18n::t;

pub struct QuickBookmarkBtn {
  state: Entity<BookViewerState>,
}

impl QuickBookmarkBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl Render for QuickBookmarkBtn {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = cx.theme();
    let state = self.state.clone();

    div()
      .id("quick-bookmark-btn")
      .w_full()
      .h(bookmarks::ACTION_BTN_HEIGHT)
      .bg(theme.foreground.opacity(0.06))
      .border_1()
      .border_color(theme.border)
      .hover(move |s| s.bg(theme.primary.opacity(0.15)).border_color(theme.primary))
      .rounded(RADIUS_MD)
      .cursor_pointer()
      .flex()
      .items_center()
      .justify_center()
      .gap_x(bookmarks::ACTION_BTN_GAP_X)
      .on_mouse_down(
        MouseButton::Left,
        cx.listener(move |_this, _, _window, cx| {
          let (book_path, page) = {
            let s = state.read(cx);
            (s.current_book.clone(), s.current_page)
          };

          if let Some(path) = book_path {
            let db = cx.db().clone();
            let now_str = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let title_str =
              t!("components.book_viewer.bookmarks.quick_title", page = page).to_string();
            let bookmark = BookMark {
              title: SharedString::from(title_str),
              content: SharedString::from(""),
              page_number: page as u32,
              time_created: SharedString::from(now_str.clone()),
              time_updated: SharedString::from(now_str),
            };
            let _ = BookBookmarks::add(&db, path, bookmark);
            cx.notify();
          }
        }),
      )
      .child(
        div()
          .text_xs()
          .font_medium()
          .text_color(theme.foreground)
          .child(t!("components.book_viewer.bookmarks.quick_btn").to_string()),
      )
  }
}
