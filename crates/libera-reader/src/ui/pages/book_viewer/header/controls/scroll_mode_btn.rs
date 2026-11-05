use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::{
  Icon, Selectable, Sizable,
  button::{Button, ButtonVariants},
};
use rust_i18n::t;

#[derive(IntoElement)]
pub struct ScrollModeBtn {
  state: Entity<BookViewerState>,
}

impl ScrollModeBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl RenderOnce for ScrollModeBtn {
  fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
    let is_continuous = self.state.read(cx).layout_mode.is_continuous();
    let state_entity = self.state.clone();

    Button::new("header-scroll-mode-btn")
      .ghost()
      .xsmall()
      .selected(is_continuous)
      .icon(Icon::new(Icon::empty()).path("pajamas--scroll-down.svg"))
      .tooltip(t!("components.book_viewer.tooltips.scroll_mode").to_string())
      .on_click(move |_, _window, cx| {
        state_entity.update(cx, |s, cx| {
          s.layout_mode.toggle_continuous();
          cx.notify();
        });
      })
  }
}
