use crate::ui::pages::book_viewer::constants::{RADIUS_SM, header};
use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::ActiveTheme;
use gpui_component::input::{Input, InputEvent, InputState};

pub struct PageNumberInput {
  state: Entity<BookViewerState>,
  input_state: Entity<InputState>,
  last_synced_page: usize,
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

      Self { state, input_state, last_synced_page: current_page, _subscription: subscription }
    })
  }
}

impl Render for PageNumberInput {
  fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let current_page = self.state.read(cx).current_page;

    // Synchronize input text with reader scroll position if it changed externally
    if self.last_synced_page != current_page {
      let new_text = current_page.to_string();
      self.input_state.update(cx, |input, cx| {
        input.set_value(new_text, window, cx);
      });
      self.last_synced_page = current_page;
    }

    let theme = cx.theme();

    div()
      .id("header-page-number-input")
      .w(header::PAGE_INPUT_WIDTH)
      .h(header::PAGE_INPUT_HEIGHT)
      .border_1()
      .border_color(theme.border)
      .hover(move |s| s.border_color(theme.primary))
      .rounded(RADIUS_SM)
      .bg(theme.background)
      .flex()
      .items_center()
      .justify_center()
      .px(header::PAGE_INPUT_PADDING_X)
      .child(
        Input::new(&self.input_state)
          .appearance(false)
          .text_sm()
          .text_color(theme.foreground)
          .w_full(),
      )
  }
}
