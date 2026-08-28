use crate::ui::pages::book_viewer::constants::viewport;
use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::ActiveTheme;
use rust_i18n::t;
pub struct PageCanvas {
  page_number: usize,
  state: Entity<BookViewerState>,
}

impl PageCanvas {
  pub fn new(page_number: usize, state: Entity<BookViewerState>) -> Self {
    Self { page_number, state }
  }
}

impl Render for PageCanvas {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = cx.theme();
    let (invert_colors, zoom_factor) = {
      let s = self.state.read(cx);
      (s.invert_colors, s.zoom_factor)
    };

    let (bg, text_color) = if invert_colors {
      // Night mode (Moon icon): Dark page background, light text
      (theme.background, theme.foreground)
    } else {
      // Day mode (Sun icon): White page background, dark text
      (gpui::white(), gpui::black().opacity(0.85))
    };

    let base_width = viewport::PAGE_BASE_WIDTH * zoom_factor;
    let base_height = viewport::PAGE_BASE_HEIGHT * zoom_factor;

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
