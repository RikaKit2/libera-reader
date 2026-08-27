use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::{
  ActiveTheme,
  input::{Input, InputEvent, InputState},
};

pub struct TtsPauseInput {
  state: Entity<BookViewerState>,
  input_state: Entity<InputState>,
  _subscription: Subscription,
}

impl TtsPauseInput {
  pub fn new(window: &mut Window, cx: &mut App, state: Entity<BookViewerState>) -> Entity<Self> {
    let pause_ms = state.read(cx).tts_pause_ms;
    let input_state = cx.new(|cx| {
      let mut input = InputState::new(window, cx);
      input.set_value(pause_ms.to_string(), window, cx);
      input
    });

    let state_clone = state.clone();
    cx.new(|cx: &mut Context<Self>| {
      let subscription = cx.subscribe_in(&input_state, window, {
        let state = state_clone.clone();
        move |_this: &mut Self, input: &Entity<InputState>, ev: &InputEvent, _window, cx| {
          if let InputEvent::Change = ev {
            let text = input.read(cx).text().to_string();
            if let Ok(val) = text.trim().parse::<u32>() {
              state.update(cx, |s, cx| {
                s.tts_pause_ms = val;
                cx.notify();
              });
            }
          }
        }
      });

      Self { state, input_state, _subscription: subscription }
    })
  }
}

impl Render for TtsPauseInput {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    div()
      .w_full()
      .flex()
      .items_center()
      .justify_between()
      .child(
        div().text_xs().text_color(cx.theme().muted_foreground).child("Пауза между предложениями:"),
      )
      .child(
        div()
          .flex()
          .items_center()
          .gap_x_1()
          .child(
            div()
              .w(px(50.0))
              .child(Input::new(&self.input_state).text_sm().bg(cx.theme().input).rounded_md()),
          )
          .child(div().text_xs().text_color(cx.theme().muted_foreground).child("мс")),
      )
  }
}
