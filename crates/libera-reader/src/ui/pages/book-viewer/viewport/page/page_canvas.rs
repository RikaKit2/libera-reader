use crate::ui::pages::book_viewer::constants::viewport;
use gpui::*;
use gpui_component::ActiveTheme;
use rust_i18n::t;

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
    let theme = cx.theme();
    let bg = if self.invert_colors { theme.foreground } else { theme.background };
    let text_color = if self.invert_colors { theme.background } else { theme.foreground };

    let base_width = viewport::PAGE_BASE_WIDTH * self.zoom_factor;
    let base_height = viewport::PAGE_BASE_HEIGHT * self.zoom_factor;

    div()
      .w(px(base_width))
      .h(px(base_height))
      .bg(bg)
      .flex()
      .flex_col()
      .items_center()
      .justify_center()
      .child(div().text_sm().text_color(text_color).child(
        t!("components.book_viewer.bookmarks.page_label", page = self.page_number).to_string(),
      ))
  }
}
