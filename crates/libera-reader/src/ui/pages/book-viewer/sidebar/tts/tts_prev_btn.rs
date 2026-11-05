use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::{
  Icon, Sizable,
  button::{Button, ButtonVariants},
};
use rust_i18n::t;

#[derive(IntoElement)]
pub struct TtsPrevBtn {
  state: Entity<BookViewerState>,
}

impl TtsPrevBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl RenderOnce for TtsPrevBtn {
  fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
    let state = self.state.clone();

    Button::new("tts-prev-btn")
      .ghost()
      .xsmall()
      .icon(Icon::new(Icon::empty()).path("heroicons--backward-20-solid.svg"))
      .tooltip(t!("components.book_viewer.tooltips.tts_prev").to_string())
      .on_click(move |_, _window, cx| {
        state.update(cx, |s, cx| {
          s.prev_page();
          cx.notify();
        });
      })
  }
}
