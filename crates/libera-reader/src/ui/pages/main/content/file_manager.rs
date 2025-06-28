use crate::ui::pages::CTX;
use gpui::{div, rgb, App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window};
use std::sync::Arc;

pub(crate) struct FileManager {
  ctx: Arc<CTX>,
}
impl FileManager {
  pub(crate) fn new(cx: &mut App, ctx: Arc<CTX>) -> Entity<Self> {
    cx.new(|_c| Self { ctx })
  }
}

impl Render for FileManager {
  fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    let theme = &self.ctx.theme.read().unwrap();
    div().w_full().h_full().flex().flex_col().text_color(rgb(theme.base_color_content)).children([
      div().bg(rgb(theme.base_100)).w_full().h_full()
    ])
  }
}
