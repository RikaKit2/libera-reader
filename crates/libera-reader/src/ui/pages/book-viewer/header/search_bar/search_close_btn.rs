use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::{
  Icon, Sizable,
  button::{Button, ButtonVariants},
};
use rust_i18n::t;

#[derive(IntoElement)]
pub struct SearchCloseBtn {
  state: Entity<BookViewerState>,
}

impl SearchCloseBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl RenderOnce for SearchCloseBtn {
  fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
    let state = self.state.clone();

    Button::new("search-close-btn")
      .ghost()
      .xsmall()
      .icon(Icon::new(Icon::empty()).path("material-symbols--close.svg"))
      .tooltip(t!("components.book_viewer.tooltips.search_close").to_string())
      .on_click(move |_, _window, cx| {
        state.update(cx, |s, cx| {
          s.toggle_search();
          cx.notify();
        });
      })
  }
}
