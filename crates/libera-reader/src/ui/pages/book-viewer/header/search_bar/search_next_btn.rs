use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::{
  Icon, Sizable,
  button::{Button, ButtonVariants},
};
use rust_i18n::t;

#[derive(IntoElement)]
pub struct SearchNextBtn {
  state: Entity<BookViewerState>,
}

impl SearchNextBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl RenderOnce for SearchNextBtn {
  fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
    let state = self.state.clone();

    Button::new("search-next-btn")
      .ghost()
      .xsmall()
      .icon(Icon::new(Icon::empty()).path("heroicons--chevron-down.svg"))
      .label(t!("components.book_viewer.search.next_btn").to_string())
      .tooltip(t!("components.book_viewer.tooltips.search_next").to_string())
      .on_click(move |_, _window, cx| {
        super::execute_search(state.clone(), cx);
      })
  }
}
