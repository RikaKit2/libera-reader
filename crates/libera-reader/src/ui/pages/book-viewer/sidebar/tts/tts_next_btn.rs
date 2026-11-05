use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::{
  Icon, Sizable,
  button::{Button, ButtonVariants},
};
use rust_i18n::t;

#[derive(IntoElement)]
pub struct TtsNextBtn {
  state: Entity<BookViewerState>,
}

impl TtsNextBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl RenderOnce for TtsNextBtn {
  fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
    let state = self.state.clone();

    Button::new("tts-next-btn")
      .ghost()
      .xsmall()
      .icon(Icon::new(Icon::empty()).path("heroicons--forward-20-solid.svg"))
      .tooltip(t!("components.book_viewer.tooltips.tts_next").to_string())
      .on_click(move |_, _window, cx| {
        state.update(cx, |s, cx| {
          s.next_page();
          cx.notify();
        });
      })
  }
}
