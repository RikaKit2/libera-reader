use crate::app_ext::AppExt;
use crate::db::models::books::bookmark::{BookBookmarks, BookMark};
use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::{ActiveTheme, Icon, IconName, Sizable, StyledExt, button::*};

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
      .p_2()
      .rounded_md()
      .bg(if is_active { cx.theme().accent } else { cx.theme().secondary })
      .hover(|s| s.bg(cx.theme().secondary_hover))
      .flex()
      .items_center()
      .justify_between()
      .gap_x_2()
      .child(
        div()
          .flex_1()
          .flex()
          .flex_col()
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
            div()
              .text_sm()
              .font_medium()
              .text_color(cx.theme().foreground)
              .child(self.bookmark.title.clone()),
          )
          .child(
            div()
              .text_xs()
              .text_color(cx.theme().muted_foreground)
              .child(format!("Страница {}", target_page)),
          ),
      )
      .child(
        Button::new(format!("del-bm-{}", self.bookmark.time_created))
          .icon(Icon::new(IconName::Delete))
          .small()
          .ghost()
          .tooltip("Удалить закладку")
          .on_click(cx.listener(move |_this, _, _window, cx| {
            if let Some(path) = &book_path {
              let db = cx.db().clone();
              let _ = BookBookmarks::remove(&db, path.clone(), &time_created);
              cx.notify();
            }
          })),
      )
  }
}
