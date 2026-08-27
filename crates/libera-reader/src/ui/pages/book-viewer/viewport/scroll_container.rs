use crate::ui::pages::book_viewer::state::BookViewerState;
use crate::ui::pages::book_viewer::viewport::page::PageView;
use gpui::*;
use gpui_component::scroll::ScrollableElement;

pub struct ScrollContainer {
  state: Entity<BookViewerState>,
}

impl ScrollContainer {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl Render for ScrollContainer {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let total_pages = self.state.read(cx).total_pages;
    let state = self.state.clone();

    div()
      .size_full()
      .overflow_y_scrollbar()
      .flex()
      .flex_col()
      .items_center()
      .py_6()
      .children((1..=total_pages).map(|p| PageView::new(p, state.clone(), cx)))
  }
}
