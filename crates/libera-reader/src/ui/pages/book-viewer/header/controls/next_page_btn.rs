use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::{
  Disableable, Icon, Sizable,
  button::{Button, ButtonVariants},
};
use rust_i18n::t;

#[derive(IntoElement)]
pub struct NextPageBtn {
  state: Entity<BookViewerState>,
}

impl NextPageBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl RenderOnce for NextPageBtn {
  fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
    let state_ref = self.state.read(cx);
    let can_go_next = state_ref.current_page < state_ref.total_pages;
    let state = self.state.clone();

    Button::new("header-next-page-btn")
      .ghost()
      .xsmall()
      .disabled(!can_go_next)
      .icon(Icon::new(Icon::empty()).path("heroicons--chevron-down.svg"))
      .tooltip(t!("components.book_viewer.tooltips.next_page").to_string())
      .on_click(move |_, _window, cx| {
        state.update(cx, |s, cx| {
          s.next_page();
          cx.notify();
        });
      })
  }
}
