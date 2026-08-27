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

pub struct PageView {
  page_number: usize,
  state: Entity<BookViewerState>,
  canvas: Entity<PageCanvas>,
  text_layer: Entity<PageTextLayer>,
  links_layer: Entity<PageLinksLayer>,
  search_layer: Entity<PageSearchLayer>,
  shadow: Entity<PageShadow>,
  badge: Entity<PageNumberBadge>,
}

impl PageView {
  pub fn new(page_number: usize, state: Entity<BookViewerState>, cx: &mut App) -> Entity<Self> {
    let (invert, zoom) = {
      let s = state.read(cx);
      (s.invert_colors, s.zoom_factor)
    };

    let canvas = cx.new(|_cx| PageCanvas::new(page_number, invert, zoom));
    let text_layer = cx.new(|_cx| PageTextLayer::new(page_number));
    let links_layer = cx.new(|_cx| PageLinksLayer::new(page_number));
    let search_layer = cx.new(|_cx| PageSearchLayer::new(page_number, state.clone()));
    let shadow = cx.new(|_cx| PageShadow::new());
    let badge = cx.new(|_cx| PageNumberBadge::new(page_number));

    cx.new(|_cx| Self {
      page_number,
      state,
      canvas,
      text_layer,
      links_layer,
      search_layer,
      shadow,
      badge,
    })
  }
}

impl Render for PageView {
  fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    div()
      .flex()
      .flex_col()
      .items_center()
      .gap_y_2()
      .my_4()
      .child(
        div()
          .relative()
          .child(self.canvas.clone())
          .child(self.text_layer.clone())
          .child(self.links_layer.clone())
          .child(self.search_layer.clone())
          .child(self.shadow.clone()),
      )
      .child(self.badge.clone())
  }
}
