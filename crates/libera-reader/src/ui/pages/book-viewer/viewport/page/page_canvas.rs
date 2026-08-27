use gpui::*;
use gpui_component::ActiveTheme;

pub struct PageCanvas {
  page_number: usize,
  invert_colors: bool,
  zoom_factor: f32,
}

impl PageCanvas {
  pub fn new(page_number: usize, invert_colors: bool, zoom_factor: f32) -> Self {
    Self { page_number, invert_colors, zoom_factor }
  }
}

impl Render for PageCanvas {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let bg = if self.invert_colors { cx.theme().background } else { hsla(0.0, 0.0, 1.0, 1.0) };

    let text_color =
      if self.invert_colors { cx.theme().foreground } else { hsla(0.0, 0.0, 0.1, 1.0) };

    let base_width = 595.0 * self.zoom_factor;
    let base_height = 842.0 * self.zoom_factor;

    div()
      .w(px(base_width))
      .h(px(base_height))
      .bg(bg)
      .flex()
      .flex_col()
      .items_center()
      .justify_center()
      .child(div().text_sm().text_color(text_color).child(format!("Страница {}", self.page_number)))
  }
}
