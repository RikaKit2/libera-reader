use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::{
  Icon,
  button::{Button, ButtonVariants},
};
use rust_i18n::t;

#[derive(IntoElement)]
pub struct TtsPlayBtn {
  state: Entity<BookViewerState>,
}

impl TtsPlayBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl RenderOnce for TtsPlayBtn {
  fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
    let playing = self.state.read(cx).tts_playing;
    let state = self.state.clone();

    Button::new("tts-play-btn")
      .primary()
      .icon(Icon::new(Icon::empty()).path(if playing {
        "ri--pause-fill.svg"
      } else {
        "ri--play-fill.svg"
      }))
      .tooltip(t!("components.book_viewer.tooltips.tts_play").to_string())
      .on_click(move |_, _window, cx| {
        state.update(cx, |s, cx| {
          s.tts_playing = !s.tts_playing;
          cx.notify();
        });
      })
  }
}
