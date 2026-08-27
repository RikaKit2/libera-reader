use crate::app_ext::AppExt;
use crate::db::models::books::bookmark::{BookBookmarks, BookMark};
use crate::ui::pages::book_viewer::state::BookViewerState;
use chrono::Local;
use gpui::*;

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
      .px(px(16.0))
      .py(px(4.0))
      .bg(rgb(0x4A4A4F))
      .hover(|s| s.bg(rgb(0x666667)))
      .rounded(px(3.0))
      .cursor_pointer()
      .text_xs()
      .text_color(rgb(0xD4D4D5))
      .child("Добавить")
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
            let bookmark = BookMark {
              title: SharedString::from(format!("Закладка на стр. {}", page)),
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
  }
}
