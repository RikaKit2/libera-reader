use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::checkbox::*;
use rust_i18n::t;

#[derive(IntoElement)]
pub struct TtsAutoTurnToggle {
  state: Entity<BookViewerState>,
}

impl TtsAutoTurnToggle {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl RenderOnce for TtsAutoTurnToggle {
  fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
    let auto_turn = self.state.read(cx).tts_auto_turn;
    let state = self.state.clone();

    Checkbox::new("tts-auto-turn-check")
      .checked(auto_turn)
      .label(t!("components.book_viewer.tts.auto_turn_label").to_string())
      .on_change(move |checked, _window, cx| {
        state.update(cx, |s, cx| {
          s.tts_auto_turn = *checked;
          cx.notify();
        });
      })
  }
}
