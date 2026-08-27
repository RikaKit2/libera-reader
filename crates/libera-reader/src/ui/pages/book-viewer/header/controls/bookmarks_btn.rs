use crate::ui::pages::book_viewer::state::{BookViewerState, SidebarTab};
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::tooltip::Tooltip;
use rust_i18n::t;
pub struct BookmarksBtn {
  state: Entity<BookViewerState>,
}

impl BookmarksBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl Render for BookmarksBtn {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let active = self.state.read(cx).active_sidebar_tab == SidebarTab::Bookmarks;
    let state = self.state.clone();

    div()
      .id("header-bookmarks-btn")
      .w(px(28.0))
      .h(px(28.0))
      .flex()
      .items_center()
      .justify_center()
      .rounded(px(3.0))
      .cursor_pointer()
      .tooltip(|window, cx| {
        Tooltip::new(t!("components.book_viewer.tooltips.bookmarks").to_string()).build(window, cx)
      })
      .when(active, |s| s.bg(rgb(0x4A4A4F)))
      .hover(|s| s.bg(rgb(0x666667)))
      .on_mouse_down(
        MouseButton::Left,
        cx.listener(move |_this, _, _window, cx| {
          state.update(cx, |s, cx| {
            s.toggle_sidebar_tab(SidebarTab::Bookmarks);
            cx.notify();
          });
        }),
      )
      .child(
        svg().path("heroicons--bookmark-20-solid.svg").size(px(22.0)).text_color(rgb(0xD4D4D5)),
      )
  }
}
