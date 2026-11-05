use crate::app_ext::AppExt;
use gpui::{
  App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Subscription,
  Window, div, px,
};
use gpui_component::input::*;
use gpui_component::{ActiveTheme, Sizable, h_flex};

const CACHE_SIZE_MIN: usize = 10;
const CACHE_SIZE_MAX: usize = 1000;
const CACHE_SIZE_STEP: usize = 10;

pub struct CacheSizeSelect {
  cache_size: usize,
  cache_input: Entity<InputState>,
  _subscriptions: Vec<Subscription>,
}

impl CacheSizeSelect {
  pub fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
    let current = cx.settings().read().image_cache_size as usize;
    let cache_input = cx
      .new(|cx| InputState::new(window, cx).default_value(current.to_string()).placeholder("1000"));

    cx.new(|cx| {
      let sub_change = cx.subscribe_in(&cache_input, window, {
        move |this: &mut CacheSizeSelect,
              state: &Entity<InputState>,
              event: &InputEvent,
              _window: &mut Window,
              cx: &mut Context<'_, CacheSizeSelect>| {
          if let InputEvent::Change = event {
            let text = state.read(cx).value();
            if let Ok(val) = text.parse::<usize>() {
              let clamped = val.clamp(CACHE_SIZE_MIN, CACHE_SIZE_MAX);
              this.cache_size = clamped;
              cx.settings_mut().set_image_cache_size(clamped as u32).unwrap();
            }
          }
        }
      });

      let sub_step = cx.subscribe_in(&cache_input, window, {
        move |this: &mut CacheSizeSelect,
              state: &Entity<InputState>,
              event: &NumberInputEvent,
              window: &mut Window,
              cx: &mut Context<'_, CacheSizeSelect>| {
          let NumberInputEvent::Step(action) = event;
          let new_val = match action {
            StepAction::Increment => (this.cache_size + CACHE_SIZE_STEP).min(CACHE_SIZE_MAX),
            StepAction::Decrement => {
              this.cache_size.saturating_sub(CACHE_SIZE_STEP).max(CACHE_SIZE_MIN)
            }
          };
          if new_val != this.cache_size {
            this.cache_size = new_val;
            cx.settings_mut().set_image_cache_size(new_val as u32).unwrap();
            state.update(cx, |s, cx| {
              s.set_value(new_val.to_string(), window, cx);
            });
          }
        }
      });

      Self { cache_size: current, cache_input, _subscriptions: vec![sub_change, sub_step] }
    })
  }
}

impl Render for CacheSizeSelect {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    div().child(
      h_flex()
        .gap_2()
        .items_center()
        .child(NumberInput::new(&self.cache_input).small().w(px(120.0)))
        .child(
          div()
            .text_sm()
            .text_color(cx.theme().muted_foreground)
            .child(format!("/ {}", CACHE_SIZE_MAX)),
        ),
    )
  }
}
