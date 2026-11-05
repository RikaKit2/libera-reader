use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::{
  Disableable, Icon, Sizable,
  button::{Button, ButtonVariants},
};
use rust_i18n::t;

#[derive(IntoElement)]
pub struct PrevPageBtn {
  state: Entity<BookViewerState>,
}

impl PrevPageBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl RenderOnce for PrevPageBtn {
  fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
    let can_go_prev = self.state.read(cx).current_page > 1;
    let state = self.state.clone();

    Button::new("header-prev-page-btn")
      .ghost()
      .xsmall()
      .disabled(!can_go_prev)
      .icon(Icon::new(Icon::empty()).path("heroicons--chevron-up.svg"))
      .tooltip(t!("components.book_viewer.tooltips.prev_page").to_string())
      .on_click(move |_, _window, cx| {
        state.update(cx, |s, cx| {
          s.prev_page();
          cx.notify();
        });
      })
  }
}
