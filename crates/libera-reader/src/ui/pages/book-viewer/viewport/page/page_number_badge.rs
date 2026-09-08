use gpui::*;
use gpui_component::ActiveTheme;

#[derive(IntoElement)]
pub struct PageNumberBadge {
  page_number: usize,
}

impl PageNumberBadge {
  pub fn new(page_number: usize) -> Self {
    Self { page_number }
  }
}

impl RenderOnce for PageNumberBadge {
  fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
    div()
      .py_1()
      .text_xs()
      .text_color(cx.theme().muted_foreground)
      .child(format!("Стр. {}", self.page_number))
  }
}
