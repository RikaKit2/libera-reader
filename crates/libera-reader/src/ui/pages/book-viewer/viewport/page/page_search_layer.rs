use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::ActiveTheme;

pub struct PageSearchLayer {
  page_number: usize,
  state: Entity<BookViewerState>,
}

impl PageSearchLayer {
  pub fn new(page_number: usize, state: Entity<BookViewerState>) -> Self {
    Self { page_number, state }
  }
}

impl Render for PageSearchLayer {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let search_state = self.state.read(cx);
    let has_match = search_state.search_results.iter().any(|hit| hit.page == self.page_number);

    if !has_match || !search_state.search_open {
      return div();
    }

    div().absolute().inset_0().border_2().border_color(cx.theme().warning)
  }
}
