pub mod controls;
pub mod header_bar;
pub mod search_bar;

pub use header_bar::HeaderBar;
pub use search_bar::SearchBar;

use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;

pub struct Header {
  header_bar: Entity<HeaderBar>,
  search_bar: Entity<SearchBar>,
}

impl Header {
  pub fn new(window: &mut Window, cx: &mut App, state: Entity<BookViewerState>) -> Entity<Self> {
    let header_bar = HeaderBar::new(window, cx, state.clone());
    let search_bar = SearchBar::new(window, cx, state);

    cx.new(|_cx| Self { header_bar, search_bar })
  }
}

impl Render for Header {
  fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    div().w_full().flex().flex_col().child(self.header_bar.clone()).child(self.search_bar.clone())
  }
}
