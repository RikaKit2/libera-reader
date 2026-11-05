use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::{
  Icon, Selectable, Sizable,
  button::{Button, ButtonVariants},
};
use rust_i18n::t;

#[derive(IntoElement)]
pub struct SearchToggleBtn {
  state: Entity<BookViewerState>,
}

impl SearchToggleBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl RenderOnce for SearchToggleBtn {
  fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
    let active = self.state.read(cx).search_open;
    let state = self.state.clone();

    Button::new("header-search-toggle-btn")
      .ghost()
      .xsmall()
      .selected(active)
      .icon(Icon::new(Icon::empty()).path("heroicons--magnifying-glass.svg"))
      .tooltip(t!("components.book_viewer.tooltips.search").to_string())
      .on_click(move |_, _window, cx| {
        state.update(cx, |s, cx| {
          s.toggle_search();
          cx.notify();
        });
      })
  }
}
