use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::{ActiveTheme, Icon, IconName, Sizable, StyledExt, button::*};

pub struct TtsSpeedSlider {
  state: Entity<BookViewerState>,
}

impl TtsSpeedSlider {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl Render for TtsSpeedSlider {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let speed = self.state.read(cx).tts_speed;
    let state = self.state.clone();

    div()
      .w_full()
      .flex()
      .flex_col()
      .gap_y_1()
      .child(
        div()
          .flex()
          .justify_between()
          .items_center()
          .child(div().text_xs().text_color(cx.theme().muted_foreground).child("Скорость речи:"))
          .child(
            div()
              .text_xs()
              .font_semibold()
              .text_color(cx.theme().foreground)
              .child(format!("{:.2}x", speed)),
          ),
      )
      .child(
        div()
          .flex()
          .items_center()
          .gap_x_2()
          .child(
            Button::new("tts-speed-dec").icon(Icon::new(IconName::Minus)).small().ghost().on_click(
              {
                let state = state.clone();
                cx.listener(move |_this, _, _window, cx| {
                  state.update(cx, |s, cx| {
                    s.tts_speed = (s.tts_speed - 0.25).max(0.5);
                    cx.notify();
                  });
                })
              },
            ),
          )
          .child(div().flex_1().h(px(6.0)).bg(cx.theme().input).rounded_full().relative().child(
            div().h_full().w(relative((speed - 0.5) / 2.5)).bg(cx.theme().primary).rounded_full(),
          ))
          .child(
            Button::new("tts-speed-inc").icon(Icon::new(IconName::Plus)).small().ghost().on_click(
              cx.listener(move |_this, _, _window, cx| {
                state.update(cx, |s, cx| {
                  s.tts_speed = (s.tts_speed + 0.25).min(3.0);
                  cx.notify();
                });
              }),
            ),
          ),
      )
  }
}
