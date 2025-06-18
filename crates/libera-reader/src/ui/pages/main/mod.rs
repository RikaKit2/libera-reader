use crate::ui::pages::Pages;
use gpui::{div, rgb, Context, IntoElement, ParentElement, Styled, Window};

pub(crate) mod content;
pub(crate) mod side_bar;
mod header;

impl Pages {
  pub fn render_main_page(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = self.settings.read().unwrap().theme.clone();
    div().bg(rgb(theme.base_100)).w_full().h_full().flex().children([
      div().h_full().child(self.render_main_page_side_bar(window, cx)),
      div().w_full().h_full(),
    ])
  }
}
