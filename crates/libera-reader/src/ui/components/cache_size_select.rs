use gpui::{
  App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div, px,
};
use gpui_component::input::*;
use gpui_component::{Sizable, gray, h_flex};
use libera_reader_core::ctx::GlobalCTX;

const CACHE_SIZE_MIN: usize = 10;
const CACHE_SIZE_MAX: usize = 1000;
const CACHE_SIZE_STEP: usize = 10;

pub struct CacheSizeSelect {
  cache_size: usize,
  cache_input: Option<Entity<InputState>>,
}

impl CacheSizeSelect {
  pub fn new(_window: &mut Window, cx: &mut App) -> Entity<Self> {
    let current = cx.ctx().settings.read().image_cache_size as usize;
    cx.new(|_cx| Self { cache_size: current, cache_input: None })
  }
}

impl Render for CacheSizeSelect {
  fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    // Lazily create InputState on first render (Window is available now)
    if self.cache_input.is_none() {
      let input = cx.new(|cx| {
        InputState::new(window, cx).default_value(self.cache_size.to_string()).placeholder("1000")
      });

      cx.subscribe_in(&input, window, {
        move |_this: &mut CacheSizeSelect,
              state: &Entity<InputState>,
              event: &InputEvent,
              _window: &mut Window,
              cx: &mut Context<'_, CacheSizeSelect>| {
          if let InputEvent::Change = event {
            let text = state.read(cx).value();
            if let Ok(val) = text.parse::<usize>() {
              let clamped = val.clamp(CACHE_SIZE_MIN, CACHE_SIZE_MAX);
              _this.cache_size = clamped;
              cx.ctx_mut().settings.set_image_cache_size(clamped as u32).unwrap();
            }
          }
        }
      })
      .detach();

      cx.subscribe_in(&input, window, {
        move |_this: &mut CacheSizeSelect,
              state: &Entity<InputState>,
              event: &NumberInputEvent,
              _window: &mut Window,
              cx: &mut Context<'_, CacheSizeSelect>| {
          let NumberInputEvent::Step(action) = event;
          let new_val = match action {
            StepAction::Increment => (_this.cache_size + CACHE_SIZE_STEP).min(CACHE_SIZE_MAX),
            StepAction::Decrement => {
              _this.cache_size.saturating_sub(CACHE_SIZE_STEP).max(CACHE_SIZE_MIN)
            }
          };
          if new_val != _this.cache_size {
            _this.cache_size = new_val;
            cx.ctx_mut().settings.set_image_cache_size(new_val as u32).unwrap();
            state.update(cx, |s, cx| {
              s.set_value(new_val.to_string(), _window, cx);
            });
          }
        }
      })
      .detach();

      self.cache_input = Some(input);
    }

    div().child(
      h_flex()
        .gap_2()
        .items_center()
        .child(NumberInput::new(self.cache_input.as_ref().unwrap()).small().w(px(120.0)))
        .child(
          div().text_sm().text_color(gray(500).opacity(0.6)).child(format!("/ {}", CACHE_SIZE_MAX)),
        ),
    )
  }
}
