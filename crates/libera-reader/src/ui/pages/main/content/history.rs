use crate::ui::components::input::TextInput;
use gpui::{div, rgb, App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window};
use libera_reader_core::ctx::GlobalCTX;

pub(crate) struct History {
  text_input: Entity<TextInput>,
}
impl History {
  pub(crate) fn new(cx: &mut App) -> Entity<Self> {
    cx.new(|c| Self { text_input: TextInput::new(c) })
  }
}

impl Render for History {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = cx.ctx().settings.theme.data();
    div().w_full().h_full().flex().flex_col().text_color(rgb(theme.base_color_content)).children([
      div().bg(rgb(theme.base_300)).w_full().h_12().flex().items_center().child(self.text_input.clone()),
      div().bg(rgb(theme.base_100)).w_full().h_full()
    ])
  }
}
