use crate::ui::pages::book_viewer::state::{BookViewerState, SidebarTab};
use gpui::*;
use gpui_component::{
  Icon, Selectable, Sizable,
  button::{Button, ButtonVariants},
};
use rust_i18n::t;

#[derive(IntoElement)]
pub struct BookmarksBtn {
  state: Entity<BookViewerState>,
}

impl BookmarksBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl RenderOnce for BookmarksBtn {
  fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
    let active = self.state.read(cx).active_sidebar_tab == SidebarTab::Bookmarks;
    let state = self.state.clone();

    Button::new("header-bookmarks-btn")
      .ghost()
      .xsmall()
      .selected(active)
      .icon(Icon::new(Icon::empty()).path("heroicons--bookmark-20-solid.svg"))
      .tooltip(t!("components.book_viewer.tooltips.bookmarks").to_string())
      .on_click(move |_, _window, cx| {
        state.update(cx, |s, cx| {
          s.toggle_sidebar_tab(SidebarTab::Bookmarks);
          cx.notify();
        });
      })
  }
}
