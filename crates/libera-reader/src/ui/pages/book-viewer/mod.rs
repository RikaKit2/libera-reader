use crate::ui::pages::Pages;
use gpui::prelude::*;
use gpui::{div, Window};

impl Pages {
  pub fn render_book_viewer(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    div()
  }
}
