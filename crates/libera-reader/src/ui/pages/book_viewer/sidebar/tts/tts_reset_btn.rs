use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::{
  Sizable,
  button::{Button, ButtonVariants},
};
use rust_i18n::t;

#[derive(IntoElement)]
pub struct TtsResetBtn {
  state: Entity<BookViewerState>,
}

impl TtsResetBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl RenderOnce for TtsResetBtn {
  fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
    let state = self.state.clone();

    Button::new("tts-reset-btn")
      .ghost()
      .xsmall()
      .label(t!("components.book_viewer.tts.reset_btn").to_string())
      .w_full()
      .on_click(move |_, _window, cx| {
        state.update(cx, |s, cx| {
          s.tts_speed = 1.0;
          s.tts_pause_ms = 150;
          s.tts_auto_turn = true;
          cx.notify();
        });
      })
  }
}
