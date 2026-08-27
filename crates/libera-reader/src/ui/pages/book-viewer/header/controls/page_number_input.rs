use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::input::{Input, InputEvent, InputState};

pub struct PageNumberInput {
  state: Entity<BookViewerState>,
  input_state: Entity<InputState>,
  _subscription: Subscription,
}

impl PageNumberInput {
  pub fn new(window: &mut Window, cx: &mut App, state: Entity<BookViewerState>) -> Entity<Self> {
    let current_page = state.read(cx).current_page;
    let input_state: Entity<InputState> = cx.new(|cx| {
      let mut input = InputState::new(window, cx);
      input.set_value(current_page.to_string(), window, cx);
      input
    });

    let state_clone = state.clone();
    cx.new(|cx: &mut Context<Self>| {
      let subscription = cx.subscribe_in(&input_state, window, {
        let state = state_clone.clone();
        move |_this: &mut Self, input: &Entity<InputState>, ev: &InputEvent, _window, cx| {
          if let InputEvent::PressEnter { .. } = ev {
            let text = input.read(cx).text().to_string();
            if let Ok(page) = text.trim().parse::<usize>() {
              state.update(cx, |s, cx| {
                s.go_to_page(page);
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

impl Render for PageNumberInput {
  fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    div()
      .id("header-page-number-input")
      .w(px(52.0))
      .h(px(28.0))
      .border_1()
      .border_color(rgb(0x5F6265))
      .hover(|s| s.border_color(rgb(0xB3B4B7)))
      .rounded(px(3.0))
      .bg(rgb(0x2A2A2E))
      .flex()
      .items_center()
      .justify_center()
      .px(px(4.0))
      .child(
        Input::new(&self.input_state)
          .appearance(false)
          .text_sm()
          .text_color(rgb(0xD4D4D5))
          .w_full(),
      )
  }
}
