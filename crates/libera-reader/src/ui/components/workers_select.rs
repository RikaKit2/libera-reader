use crate::ctx::GlobalCTX;
use gpui::*;
use gpui_component::gray;
use gpui_component::input::*;
use gpui_component::*;
use rust_i18n::t;

pub struct WorkersSelect {
  input: Entity<InputState>,
}

impl WorkersSelect {
  pub fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
    let current = cx.ctx().settings.read().workers_num;
    let max_workers: u32 =
      std::thread::available_parallelism().map(|n| n.get() as u32).unwrap_or(4);

    let input =
      cx.new(|cx| InputState::new(window, cx).default_value(current.to_string()).placeholder("1"));

    cx.new(|cx| {
      cx.subscribe_in(&input, window, {
        move |_this: &mut WorkersSelect,
              state: &Entity<InputState>,
              event: &gpui_component::input::InputEvent,
              _window: &mut Window,
              cx: &mut Context<'_, WorkersSelect>| {
          if let gpui_component::input::InputEvent::Change = event {
            let text = state.read(cx).value();
            if let Ok(val) = text.parse::<u32>() {
              let clamped = val.clamp(1, max_workers as u32);
              cx.ctx_mut().settings.set_workers_num(clamped).unwrap();
            }
          }
        }
      })
      .detach();

      cx.subscribe_in(&input, window, {
        move |_this: &mut WorkersSelect,
              state: &Entity<InputState>,
              event: &NumberInputEvent,
              _window: &mut Window,
              cx: &mut Context<'_, WorkersSelect>| {
          let current = cx.ctx().settings.read().workers_num;
          match event {
            NumberInputEvent::Step(StepAction::Increment) => {
              if current < max_workers as u32 {
                let new_val = current + 1;
                cx.ctx_mut().settings.set_workers_num(new_val).unwrap();
                state.update(cx, |input, cx| {
                  input.set_value(new_val.to_string(), _window, cx);
                });
              }
            }
            NumberInputEvent::Step(StepAction::Decrement) => {
              if current > 1 {
                let new_val = current - 1;
                cx.ctx_mut().settings.set_workers_num(new_val).unwrap();
                state.update(cx, |input, cx| {
                  input.set_value(new_val.to_string(), _window, cx);
                });
              }
            }
          }
        }
      })
      .detach();

      Self { input }
    })
  }
}

impl Render for WorkersSelect {
  fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    div().child(
      h_flex()
        .gap_2()
        .items_center()
        .child(NumberInput::new(&self.input).small().w(px(100.0)))
        .child(div().text_sm().text_color(gray(500).opacity(0.6)).child(format!(
          "/ {} {}",
          std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4),
          t!("components.workers_select.max_label")
        ))),
    )
  }
}
