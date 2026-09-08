pub mod page_canvas;
pub mod page_links_layer;
pub mod page_number_badge;
pub mod page_search_layer;
pub mod page_shadow;
pub mod page_text_layer;

pub use page_canvas::PageCanvas;
pub use page_links_layer::PageLinksLayer;
pub use page_number_badge::PageNumberBadge;
pub use page_search_layer::PageSearchLayer;
pub use page_shadow::PageShadow;
pub use page_text_layer::PageTextLayer;

use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_base::TextSelectionHandle;
use mutool::{PageLink, PageStructuredText};
use std::sync::Arc;

#[derive(IntoElement)]
pub struct PageView {
  page_number: usize,
  state: Entity<BookViewerState>,
  image: Option<Arc<RenderImage>>,
  is_image_loading: bool,
  stext: Option<Arc<PageStructuredText>>,
  links: Option<Arc<Vec<PageLink>>>,
  selection_handle: TextSelectionHandle,
}

impl PageView {
  pub fn new(
    page_number: usize, state: Entity<BookViewerState>, image: Option<Arc<RenderImage>>,
    is_image_loading: bool, stext: Option<Arc<PageStructuredText>>,
    links: Option<Arc<Vec<PageLink>>>, selection_handle: TextSelectionHandle,
  ) -> Self {
    Self { page_number, state, image, is_image_loading, stext, links, selection_handle }
  }
}

impl RenderOnce for PageView {
  fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
    let (zoom_factor, page_dims) = {
      let s = self.state.read(cx);
      (s.zoom_factor, s.page_size(self.page_number))
    };

    let width = page_dims.width * zoom_factor;
    let height = page_dims.height * zoom_factor;

    let canvas =
      PageCanvas::new(self.page_number, self.state.clone(), self.image, self.is_image_loading);
    let shadow = PageShadow::new();
    let links_layer = PageLinksLayer::new(self.page_number, self.state.clone(), self.links);
    let search_layer =
      PageSearchLayer::new(self.page_number, self.state.clone(), self.stext.clone());
    let text_layer = PageTextLayer::new(
      self.page_number,
      zoom_factor,
      self.selection_handle,
      self.stext,
      width,
      height,
      window,
    );
    let badge = PageNumberBadge::new(self.page_number);

    div()
      .flex()
      .flex_col()
      .items_center()
      .gap_y_2()
      .child(
        div()
          .w(px(width))
          .h(px(height))
          .relative()
          .child(canvas)
          .child(search_layer)
          .child(links_layer)
          .child(text_layer)
          .child(shadow),
      )
      .child(badge)
  }
}
