use crate::ui::components::input::TextInput;
use crate::ui::pages::CTX;
use gpui::{div, rgb, App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window};
use std::sync::Arc;

pub(crate) struct Library {
  ctx: Arc<CTX>,
  text_input: Entity<TextInput>,
}
impl Library {
  pub(crate) fn new(cx: &mut App, ctx: Arc<CTX>) -> Entity<Self> {
    let settings = ctx.settings.clone();
    cx.new(|c| Self { ctx, text_input: TextInput::new(c, "Search".into(), settings) })
  }
}

impl Render for Library {
  fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    let theme = self.ctx.settings.read().unwrap().theme.clone();
    div().w_full().h_full().flex().flex_col().text_color(rgb(theme.base_color_content)).children([
      div().bg(rgb(theme.base_300)).w_full().h_12().flex().items_center().child(self.text_input.clone()),
      div().bg(rgb(theme.base_100)).w_full().h_full()
    ])
  }
}
