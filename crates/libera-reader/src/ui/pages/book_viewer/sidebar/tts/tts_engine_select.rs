use crate::ui::pages::book_viewer::state::{BookViewerState, TtsConfigProvider, TtsEngineConfig};
use gpui::*;
use gpui_component::{
  ActiveTheme, IndexPath, Sizable,
  select::{SearchableVec, Select, SelectEvent, SelectState},
};
use rust_i18n::t;

pub struct TtsEngineSelect {
  state: Entity<BookViewerState>,
  select_state: Entity<SelectState<SearchableVec<TtsEngineConfig>>>,
  _subscription: Subscription,
}

impl TtsEngineSelect {
  pub fn new(window: &mut Window, cx: &mut App, state: Entity<BookViewerState>) -> Entity<Self> {
    let engines = TtsConfigProvider::available_engines();
    let current_engine = state.read(cx).tts_engine.clone();
    let initial_idx = engines
      .iter()
      .position(|e| e.id == current_engine)
      .map(|i| IndexPath::default().row(i))
      .or(Some(IndexPath::default().row(0)));

    let searchable = SearchableVec::new(engines);
    let select_state =
      cx.new(|cx| SelectState::new(searchable, initial_idx, window, cx).searchable(false));

    let state_clone = state.clone();
    cx.new(|cx: &mut Context<Self>| {
      let subscription = cx.subscribe_in(&select_state, window, {
        let state = state_clone.clone();
        move |_this: &mut Self,
              _select: &Entity<SelectState<SearchableVec<TtsEngineConfig>>>,
              event: &SelectEvent<SearchableVec<TtsEngineConfig>>,
              _window: &mut Window,
              cx: &mut Context<'_, Self>| {
          if let SelectEvent::Confirm(Some(engine)) = event {
            state.update(cx, |s, cx| {
              s.tts_engine = engine.id.clone();
              cx.notify();
            });
          }
        }
      });

      Self { state, select_state, _subscription: subscription }
    })
  }
}

impl Render for TtsEngineSelect {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = cx.theme();

    div()
      .w_full()
      .flex()
      .flex_col()
      .gap_y_1()
      .child(
        div()
          .text_xs()
          .text_color(theme.foreground)
          .child(t!("components.book_viewer.tts.engine_label").to_string()),
      )
      .child(Select::new(&self.select_state).small().bg(theme.input).w_full())
  }
}
