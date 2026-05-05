use gpui::{
  App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div,
};
use gpui_component::ActiveTheme;

pub(crate) struct Stats {}
impl Stats {
  pub(crate) fn new(cx: &mut App) -> Entity<Self> {
    cx.new(|_c| Self {})
  }
}

impl Render for Stats {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let text_cover = cx.theme().foreground;
    let bg_color = cx.theme().background;
    div()
      .w_full()
      .h_full()
      .flex()
      .flex_col()
      .text_color(text_cover)
      .children([div().bg(bg_color).w_full().h_full()])
  }
}
