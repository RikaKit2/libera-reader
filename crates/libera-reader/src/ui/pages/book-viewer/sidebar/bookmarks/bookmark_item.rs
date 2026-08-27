use crate::app_ext::AppExt;
use crate::db::models::books::bookmark::{BookBookmarks, BookMark};
use crate::ui::pages::book_viewer::constants::{RADIUS_MD, RADIUS_SM, bookmarks};
use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::ActiveTheme;
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
    let theme = cx.theme();
    let target_page = self.bookmark.page_number as usize;
    let state = self.state.clone();
    let current_page = self.state.read(cx).current_page;
    let is_active = current_page == target_page;

    let time_created = self.bookmark.time_created.clone();
    let book_path = self.state.read(cx).current_book.clone();

    div()
      .w_full()
      .p(bookmarks::ITEM_PADDING)
      .rounded(RADIUS_MD)
      .bg(if is_active { theme.primary.opacity(0.15) } else { theme.foreground.opacity(0.04) })
      .border_1()
      .border_color(if is_active { theme.primary } else { theme.border })
      .hover(move |s| s.bg(theme.foreground.opacity(0.08)).border_color(theme.primary.opacity(0.5)))
      .flex()
      .items_center()
      .justify_between()
      .gap_x(bookmarks::ITEM_GAP_X)
      .child(
        div()
          .flex_1()
          .flex()
          .items_center()
          .gap_x(bookmarks::ITEM_GAP_X)
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
              .size(bookmarks::ITEM_ICON_SIZE)
              .text_color(if is_active { theme.primary } else { theme.muted_foreground }),
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
                  .text_color(theme.foreground)
                  .child(self.bookmark.title.clone()),
              )
              .child(div().text_xs().text_color(theme.muted_foreground).child(
                t!("components.book_viewer.bookmarks.page_label", page = target_page).to_string(),
              )),
          ),
      )
      .child(
        div()
          .id(format!("del-bm-{}", self.bookmark.time_created))
          .w(bookmarks::DEL_BTN_SIZE)
          .h(bookmarks::DEL_BTN_SIZE)
          .flex()
          .items_center()
          .justify_center()
          .rounded(RADIUS_SM)
          .cursor_pointer()
          .tooltip(|window, cx| {
            Tooltip::new(t!("components.book_viewer.tooltips.delete_bookmark").to_string())
              .build(window, cx)
          })
          .hover(move |s| s.bg(theme.foreground.opacity(0.12)))
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
            svg()
              .path("material-symbols--close.svg")
              .size(bookmarks::DEL_BTN_ICON_SIZE)
              .text_color(theme.muted_foreground),
          ),
      )
  }
}
