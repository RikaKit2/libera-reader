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
  play_btn: Entity<TtsPlayBtn>,
  prev_btn: Entity<TtsPrevBtn>,
  next_btn: Entity<TtsNextBtn>,
  speed_slider: Entity<TtsSpeedSlider>,
  pause_input: Entity<TtsPauseInput>,
  auto_turn_toggle: Entity<TtsAutoTurnToggle>,
  reset_btn: Entity<TtsResetBtn>,
}

impl TtsView {
  pub fn new(window: &mut Window, cx: &mut App, state: Entity<BookViewerState>) -> Entity<Self> {
    let engine_select = TtsEngineSelect::new(window, cx, state.clone());
    let voice_select = TtsVoiceSelect::new(window, cx, state.clone());
    let play_btn = cx.new(|_cx| TtsPlayBtn::new(state.clone()));
    let prev_btn = cx.new(|_cx| TtsPrevBtn::new(state.clone()));
    let next_btn = cx.new(|_cx| TtsNextBtn::new(state.clone()));
    let speed_slider = cx.new(|_cx| TtsSpeedSlider::new(state.clone()));
    let pause_input = TtsPauseInput::new(window, cx, state.clone());
    let auto_turn_toggle = cx.new(|_cx| TtsAutoTurnToggle::new(state.clone()));
    let reset_btn = cx.new(|_cx| TtsResetBtn::new(state.clone()));

    cx.new(|_cx| Self {
      state,
      engine_select,
      voice_select,
      play_btn,
      prev_btn,
      next_btn,
      speed_slider,
      pause_input,
      auto_turn_toggle,
      reset_btn,
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
          .child(self.prev_btn.clone())
          .child(self.play_btn.clone())
          .child(self.next_btn.clone()),
      )
      // Engine & Voice selection
      .child(self.engine_select.clone())
      .child(self.voice_select.clone())
      // Speed slider
      .child(self.speed_slider.clone())
      // Pause input
      .child(self.pause_input.clone())
      // Auto turn
      .child(self.auto_turn_toggle.clone())
      // Reset button
      .child(div().pt_2().child(self.reset_btn.clone()))
  }
}
