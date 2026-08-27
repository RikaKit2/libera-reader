pub mod thumbnail_card;

pub use thumbnail_card::ThumbnailCard;

use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::scroll::ScrollableElement;

pub struct ThumbnailsView {
  state: Entity<BookViewerState>,
}

impl ThumbnailsView {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl Render for ThumbnailsView {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let total_pages = self.state.read(cx).total_pages;
    let state = self.state.clone();

    div()
      .size_full()
      .overflow_y_scrollbar()
      .p_3()
      .flex()
      .flex_wrap()
      .justify_center()
      .gap_3()
      .children((1..=total_pages).map(|page| cx.new(|_cx| ThumbnailCard::new(page, state.clone()))))
  }
}
