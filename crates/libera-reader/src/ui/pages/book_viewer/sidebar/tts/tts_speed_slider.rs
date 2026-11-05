use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::slider::{Slider, SliderEvent, SliderState};
use gpui_component::{ActiveTheme, Icon, IconName, Sizable, StyledExt, button::*};
use rust_i18n::t;

pub struct TtsSpeedSlider {
  state: Entity<BookViewerState>,
  slider_state: Entity<SliderState>,
  _subscriptions: Vec<Subscription>,
}

impl TtsSpeedSlider {
  pub fn new(window: &mut Window, cx: &mut App, state: Entity<BookViewerState>) -> Entity<Self> {
    let speed = state.read(cx).tts_speed;
    let slider_state =
      cx.new(|_| SliderState::new().min(0.5).max(3.0).step(0.25).default_value(speed));

    let state_clone = state.clone();
    let slider_clone = slider_state.clone();
    cx.new(|cx| {
      let sub_slider = cx.subscribe_in(&slider_state, window, {
        let state = state_clone.clone();
        move |_this: &mut Self, _slider, event: &SliderEvent, _window, cx| match event {
          SliderEvent::Change(val) | SliderEvent::Release(val) => {
            let new_speed = val.start();
            state.update(cx, |s, cx| {
              if (s.tts_speed - new_speed).abs() > 0.01 {
                s.tts_speed = new_speed;
                cx.notify();
              }
            });
          }
        }
      });

      let sub_state = cx.observe_in(&state, window, move |_this, state, window, cx| {
        let current_speed = state.read(cx).tts_speed;
        let current_slider_val = slider_clone.read(cx).value().start();
        if (current_speed - current_slider_val).abs() > 0.01 {
          slider_clone.update(cx, |s, cx| {
            s.set_value(current_speed, window, cx);
          });
        }
      });

      Self { state, slider_state, _subscriptions: vec![sub_slider, sub_state] }
    })
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
          .child(
            div()
              .text_xs()
              .text_color(cx.theme().muted_foreground)
              .child(t!("components.book_viewer.tts.speed_label").to_string()),
          )
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
                move |_ev, _window, cx| {
                  state.update(cx, |s, cx| {
                    s.tts_speed = (s.tts_speed - 0.25).max(0.5);
                    cx.notify();
                  });
                }
              },
            ),
          )
          .child(Slider::new(&self.slider_state).flex_1())
          .child(
            Button::new("tts-speed-inc").icon(Icon::new(IconName::Plus)).small().ghost().on_click(
              move |_ev, _window, cx| {
                state.update(cx, |s, cx| {
                  s.tts_speed = (s.tts_speed + 0.25).min(3.0);
                  cx.notify();
                });
              },
            ),
          ),
      )
  }
}
