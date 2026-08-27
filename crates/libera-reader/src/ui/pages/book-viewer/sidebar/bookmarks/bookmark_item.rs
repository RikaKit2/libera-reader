use crate::app_ext::AppExt;
use crate::db::models::books::bookmark::{BookBookmarks, BookMark};
use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::StyledExt;
use gpui_component::tooltip::Tooltip;
use rust_i18n::t;
pub struct BookmarkItem {
  bookmark: BookMark,
  state: Entity<BookViewerState>,
}

impl BookmarkItem {
  pub fn new(bookmark: BookMark, state: Entity<BookViewerState>) -> Self {
    Self { bookmark, state }
  }
}

impl Render for BookmarkItem {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let target_page = self.bookmark.page_number as usize;
    let state = self.state.clone();
    let current_page = self.state.read(cx).current_page;
    let is_active = current_page == target_page;

    let time_created = self.bookmark.time_created.clone();
    let book_path = self.state.read(cx).current_book.clone();

    div()
      .w_full()
      .p(px(8.0))
      .rounded(px(4.0))
      .bg(if is_active { rgb(0x4A4A4F) } else { rgb(0x333338) })
      .border_1()
      .border_color(if is_active { rgb(0x707078) } else { rgb(0x3E3E44) })
      .hover(|s| s.bg(rgb(0x404046)).border_color(rgb(0x5A5A62)))
      .flex()
      .items_center()
      .justify_between()
      .gap_x(px(8.0))
      .child(
        div()
          .flex_1()
          .flex()
          .items_center()
          .gap_x(px(8.0))
          .cursor_pointer()
          .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |_this, _, _window, cx| {
              state.update(cx, |s, cx| {
                s.go_to_page(target_page);
                cx.notify();
              });
            }),
          )
          .child(
            svg()
              .path("heroicons--bookmark-20-solid.svg")
              .size(px(16.0))
              .text_color(if is_active { rgb(0xFFD166) } else { rgb(0xA0A0A5) }),
          )
          .child(
            div()
              .flex_1()
              .flex()
              .flex_col()
              .child(
                div()
                  .text_xs()
                  .font_medium()
                  .text_color(rgb(0xE0E0E5))
                  .child(self.bookmark.title.clone()),
              )
              .child(div().text_xs().text_color(rgb(0x9E9EA4)).child(
                t!("components.book_viewer.bookmarks.page_label", page = target_page).to_string(),
              )),
          ),
      )
      .child(
        div()
          .id(format!("del-bm-{}", self.bookmark.time_created))
          .w(px(22.0))
          .h(px(22.0))
          .flex()
          .items_center()
          .justify_center()
          .rounded(px(3.0))
          .cursor_pointer()
          .tooltip(|window, cx| {
            Tooltip::new(t!("components.book_viewer.tooltips.delete_bookmark").to_string())
              .build(window, cx)
          })
          .hover(|s| s.bg(rgb(0x5A5A62)))
          .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |_this, _, _window, cx| {
              if let Some(path) = &book_path {
                let db = cx.db().clone();
                let _ = BookBookmarks::remove(&db, path.clone(), &time_created);
                cx.notify();
              }
            }),
          )
          .child(
            svg().path("material-symbols--close.svg").size(px(14.0)).text_color(rgb(0xA0A0A5)),
          ),
      )
  }
}
