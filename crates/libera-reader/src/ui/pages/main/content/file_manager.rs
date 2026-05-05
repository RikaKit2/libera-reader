use gpui::{
  App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div,
};
use gpui_component::ActiveTheme;

pub(crate) struct FileManager {}
impl FileManager {
  pub(crate) fn new(cx: &mut App) -> Entity<Self> {
    cx.new(|_c| Self {})
  }
}

impl Render for FileManager {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    div()
      .w_full()
      .h_full()
      .flex()
      .flex_col()
      .text_color(cx.theme().foreground)
      .children([div().bg(cx.theme().background).w_full().h_full()])
  }
}
