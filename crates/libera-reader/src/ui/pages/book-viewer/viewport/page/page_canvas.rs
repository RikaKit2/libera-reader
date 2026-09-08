use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::ActiveTheme;
use rust_i18n::t;
use std::sync::Arc;

#[derive(IntoElement)]
pub struct PageCanvas {
  page_number: usize,
  state: Entity<BookViewerState>,
  image: Option<Arc<RenderImage>>,
  is_loading: bool,
}

impl PageCanvas {
  pub fn new(
    page_number: usize, state: Entity<BookViewerState>, image: Option<Arc<RenderImage>>,
    is_loading: bool,
  ) -> Self {
    Self { page_number, state, image, is_loading }
  }
}

impl RenderOnce for PageCanvas {
  fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
    let theme = cx.theme();
    let (invert_colors, zoom_factor, page_dims) = {
      let s = self.state.read(cx);
      (s.invert_colors, s.zoom_factor, s.page_size(self.page_number))
    };

    let (bg, text_color) = if invert_colors {
      (theme.background, theme.foreground)
    } else {
      (gpui::white(), gpui::black().opacity(0.85))
    };

    let width = page_dims.width * zoom_factor;
    let height = page_dims.height * zoom_factor;

    let mut container = div().w(px(width)).h(px(height)).bg(bg).relative().overflow_hidden();

    if let Some(img) = &self.image {
      let image_el =
        gpui::img(ImageSource::Render(img.clone())).w_full().h_full().object_fit(ObjectFit::Fill);

      container = container.child(image_el);
    } else {
      // Placeholder while loading
      container = container.flex().flex_col().items_center().justify_center().child(
        div().text_sm().text_color(text_color).child(
          t!("components.book_viewer.bookmarks.page_label", page = self.page_number).to_string(),
        ),
      );
    }

    container
  }
}
