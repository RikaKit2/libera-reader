use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;

pub struct PageCounter {
  state: Entity<BookViewerState>,
}

impl PageCounter {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl Render for PageCounter {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let state_ref = self.state.read(cx);
    let current = state_ref.current_page;
    let total = state_ref.total_pages;

    div().text_sm().text_color(rgb(0xD4D4D5)).child(format!("({} of {})", current, total))
  }
}
