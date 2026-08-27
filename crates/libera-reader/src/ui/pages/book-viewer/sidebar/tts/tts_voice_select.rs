use crate::ui::pages::book_viewer::state::{BookViewerState, TtsConfigProvider, TtsVoiceConfig};
use gpui::*;
use gpui_component::{
  ActiveTheme, IndexPath, Sizable,
  select::{SearchableVec, Select, SelectEvent, SelectState},
};
use rust_i18n::t;
pub struct TtsVoiceSelect {
  state: Entity<BookViewerState>,
  select_state: Entity<SelectState<SearchableVec<TtsVoiceConfig>>>,
  _subscription: Subscription,
}

impl TtsVoiceSelect {
  pub fn new(window: &mut Window, cx: &mut App, state: Entity<BookViewerState>) -> Entity<Self> {
    let current_engine = state.read(cx).tts_engine.clone();
    let voices = TtsConfigProvider::available_voices(&current_engine);
    let current_voice = state.read(cx).tts_voice.clone();
    let initial_idx = voices
      .iter()
      .position(|v| v.id == current_voice)
      .map(|i| IndexPath::default().row(i))
      .or(Some(IndexPath::default().row(0)));

    let searchable = SearchableVec::new(voices);
    let select_state =
      cx.new(|cx| SelectState::new(searchable, initial_idx, window, cx).searchable(false));

    let state_clone = state.clone();
    cx.new(|cx: &mut Context<Self>| {
      let subscription = cx.subscribe_in(&select_state, window, {
        let state = state_clone.clone();
        move |_this: &mut Self,
              _select: &Entity<SelectState<SearchableVec<TtsVoiceConfig>>>,
              event: &SelectEvent<SearchableVec<TtsVoiceConfig>>,
              _window: &mut Window,
              cx: &mut Context<'_, Self>| {
          if let SelectEvent::Confirm(Some(voice)) = event {
            state.update(cx, |s, cx| {
              s.tts_voice = voice.id.clone();
              cx.notify();
            });
          }
        }
      });

      Self { state, select_state, _subscription: subscription }
    })
  }
}

impl Render for TtsVoiceSelect {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    div()
      .w_full()
      .flex()
      .flex_col()
      .gap_y_1()
      .child(
        div()
          .text_xs()
          .text_color(rgb(0xD4D4D5))
          .child(t!("components.book_viewer.tts.voice_label").to_string()),
      )
      .child(Select::new(&self.select_state).small().bg(cx.theme().input).w_full())
  }
}
