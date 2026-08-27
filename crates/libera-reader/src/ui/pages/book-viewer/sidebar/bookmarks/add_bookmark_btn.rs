use crate::app_ext::AppExt;
use crate::db::models::books::bookmark::{BookBookmarks, BookMark};
use crate::ui::pages::book_viewer::state::BookViewerState;
use chrono::Local;
use gpui::*;
use gpui_component::StyledExt;
use rust_i18n::t;

pub struct AddBookmarkBtn {
  state: Entity<BookViewerState>,
}

impl AddBookmarkBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl Render for AddBookmarkBtn {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let state = self.state.clone();
    div()
      .id("add-bookmark-btn")
      .w_full()
      .h(px(30.0))
      .bg(rgb(0x38383D))
      .border_1()
      .border_color(rgb(0x4A4A4F))
      .hover(|s| s.bg(rgb(0x4A4A4F)).border_color(rgb(0x666667)))
      .rounded(px(4.0))
      .cursor_pointer()
      .flex()
      .items_center()
      .justify_center()
      .gap_x(px(6.0))
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
              t!("components.book_viewer.bookmarks.default_title", page = page).to_string();
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
        svg().path("heroicons--bookmark-20-solid.svg").size(px(14.0)).text_color(rgb(0xD4D4D5)),
      )
      .child(
        div()
          .text_xs()
          .font_medium()
          .text_color(rgb(0xD4D4D5))
          .child(t!("components.book_viewer.bookmarks.add_btn").to_string()),
      )
  }
}
