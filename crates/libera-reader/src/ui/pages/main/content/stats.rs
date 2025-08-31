use gpui::{App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div, rgb};
use libera_reader_core::ctx::GlobalCTX;

pub(crate) struct Stats {}
impl Stats {
  pub(crate) fn new(cx: &mut App) -> Entity<Self> {
    cx.new(|_c| Self {})
  }
}

impl Render for Stats {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = cx.ctx().settings.read().theme.data();
    div().w_full().h_full().flex().flex_col().text_color(rgb(theme.base_color_content)).children([div().bg(rgb(theme.base_100)).w_full().h_full()])
  }
}
