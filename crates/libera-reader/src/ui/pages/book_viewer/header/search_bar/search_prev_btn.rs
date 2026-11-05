use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::{
  Icon, Sizable,
  button::{Button, ButtonVariants},
};
use rust_i18n::t;

#[derive(IntoElement)]
pub struct SearchPrevBtn {
  state: Entity<BookViewerState>,
}

impl SearchPrevBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl RenderOnce for SearchPrevBtn {
  fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
    let state = self.state.clone();

    Button::new("search-prev-btn")
      .ghost()
      .xsmall()
      .icon(Icon::new(Icon::empty()).path("heroicons--chevron-up.svg"))
      .label(t!("components.book_viewer.search.prev_btn").to_string())
      .tooltip(t!("components.book_viewer.tooltips.search_prev").to_string())
      .on_click(move |_, _window, cx| {
        state.update(cx, |s, cx| {
          s.prev_search_match();
          cx.notify();
        });
      })
  }
}
