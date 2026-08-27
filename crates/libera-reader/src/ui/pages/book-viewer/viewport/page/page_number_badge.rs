use gpui::*;
use gpui_component::ActiveTheme;

pub struct PageNumberBadge {
  page_number: usize,
}

impl PageNumberBadge {
  pub fn new(page_number: usize) -> Self {
    Self { page_number }
  }
}

impl Render for PageNumberBadge {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    div()
      .py_1()
      .text_xs()
      .text_color(cx.theme().muted_foreground)
      .child(format!("Стр. {}", self.page_number))
  }
}
