pub mod tts_auto_turn_toggle;
pub mod tts_engine_select;
pub mod tts_next_btn;
pub mod tts_pause_input;
pub mod tts_play_btn;
pub mod tts_prev_btn;
pub mod tts_reset_btn;
pub mod tts_speed_slider;
pub mod tts_voice_select;

pub use tts_auto_turn_toggle::TtsAutoTurnToggle;
pub use tts_engine_select::TtsEngineSelect;
pub use tts_next_btn::TtsNextBtn;
pub use tts_pause_input::TtsPauseInput;
pub use tts_play_btn::TtsPlayBtn;
pub use tts_prev_btn::TtsPrevBtn;
pub use tts_reset_btn::TtsResetBtn;
pub use tts_speed_slider::TtsSpeedSlider;
pub use tts_voice_select::TtsVoiceSelect;

use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::scroll::ScrollableElement;

pub struct TtsView {
  state: Entity<BookViewerState>,
  engine_select: Entity<TtsEngineSelect>,
  voice_select: Entity<TtsVoiceSelect>,
  speed_slider: Entity<TtsSpeedSlider>,
  pause_input: Entity<TtsPauseInput>,
  _subscription: Subscription,
}

impl TtsView {
  pub fn new(window: &mut Window, cx: &mut App, state: Entity<BookViewerState>) -> Entity<Self> {
    let engine_select = TtsEngineSelect::new(window, cx, state.clone());
    let voice_select = TtsVoiceSelect::new(window, cx, state.clone());
    let speed_slider = TtsSpeedSlider::new(window, cx, state.clone());
    let pause_input = TtsPauseInput::new(window, cx, state.clone());

    cx.new(|cx| {
      let sub = cx.observe(&state, |_, _, cx| cx.notify());
      Self { state, engine_select, voice_select, speed_slider, pause_input, _subscription: sub }
    })
  }
}

impl Render for TtsView {
  fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    div()
      .size_full()
      .overflow_y_scrollbar()
      .p_3()
      .flex()
      .flex_col()
      .gap_y_4()
      // Player controls
      .child(
        div()
          .flex()
          .items_center()
          .justify_center()
          .gap_x_3()
          .child(TtsPrevBtn::new(self.state.clone()))
          .child(TtsPlayBtn::new(self.state.clone()))
          .child(TtsNextBtn::new(self.state.clone())),
      )
      // Engine & Voice selection
      .child(self.engine_select.clone())
      .child(self.voice_select.clone())
      // Speed slider
      .child(self.speed_slider.clone())
      // Pause input
      .child(self.pause_input.clone())
      // Auto turn
      .child(TtsAutoTurnToggle::new(self.state.clone()))
      // Reset button
      .child(div().pt_2().child(TtsResetBtn::new(self.state.clone())))
  }
}
