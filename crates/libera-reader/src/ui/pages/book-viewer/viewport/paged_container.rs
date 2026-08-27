use crate::ui::pages::book_viewer::state::BookViewerState;
use crate::ui::pages::book_viewer::viewport::page::PageView;
use gpui::*;

pub struct PagedContainer {
  state: Entity<BookViewerState>,
}

impl PagedContainer {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl Render for PagedContainer {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let current_page = self.state.read(cx).current_page;
    let state = self.state.clone();

    div().size_full().flex().flex_col().items_center().justify_center().p_4().child(PageView::new(
      current_page,
      state,
      cx,
    ))
  }
}
