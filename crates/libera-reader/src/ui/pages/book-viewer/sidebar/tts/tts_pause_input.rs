use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::input::{Input, InputEvent, InputState};
use rust_i18n::t;
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
  fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    div()
      .w_full()
      .flex()
      .flex_col()
      .gap_y(px(4.0))
      .child(
        div()
          .text_xs()
          .text_color(rgb(0xD4D4D5))
          .child(t!("components.book_viewer.tts.pause_label").to_string()),
      )
      .child(
        div()
          .flex()
          .items_center()
          .gap_x(px(6.0))
          .child(
            div()
              .w(px(64.0))
              .h(px(26.0))
              .border_1()
              .border_color(rgb(0x5F6265))
              .hover(|s| s.border_color(rgb(0xB3B4B7)))
              .rounded(px(3.0))
              .bg(rgb(0x2A2A2E))
              .flex()
              .items_center()
              .px(px(4.0))
              .child(
                Input::new(&self.input_state)
                  .appearance(false)
                  .text_sm()
                  .text_color(rgb(0xD4D4D5))
                  .w_full(),
              ),
          )
          .child(
            div()
              .text_xs()
              .text_color(rgb(0x9E9EA4))
              .child(t!("components.book_viewer.tts.pause_unit").to_string()),
          ),
      )
  }
}
