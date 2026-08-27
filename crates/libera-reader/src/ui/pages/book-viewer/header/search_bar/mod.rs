pub mod search_close_btn;
pub mod search_counter;
pub mod search_input;
pub mod search_next_btn;
pub mod search_prev_btn;

pub use search_close_btn::SearchCloseBtn;
pub use search_counter::SearchCounter;
pub use search_input::SearchInput;
pub use search_next_btn::SearchNextBtn;
pub use search_prev_btn::SearchPrevBtn;

use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;

pub struct SearchBar {
  state: Entity<BookViewerState>,
  input: Entity<SearchInput>,
  prev_btn: Entity<SearchPrevBtn>,
  next_btn: Entity<SearchNextBtn>,
  counter: Entity<SearchCounter>,
  close_btn: Entity<SearchCloseBtn>,
}

impl SearchBar {
  pub fn new(window: &mut Window, cx: &mut App, state: Entity<BookViewerState>) -> Entity<Self> {
    let input = SearchInput::new(window, cx, state.clone());
    let prev_btn = cx.new(|_cx| SearchPrevBtn::new(state.clone()));
    let next_btn = cx.new(|_cx| SearchNextBtn::new(state.clone()));
    let counter = cx.new(|_cx| SearchCounter::new(state.clone()));
    let close_btn = cx.new(|_cx| SearchCloseBtn::new(state.clone()));

    cx.new(|_cx| Self { state, input, prev_btn, next_btn, counter, close_btn })
  }
}

impl Render for SearchBar {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let is_open = self.state.read(cx).search_open;

    if !is_open {
      return div();
    }

    div()
      .w_full()
      .h(px(32.0))
      .bg(rgb(0x2A2A2E))
      .flex()
      .items_center()
      .px(px(4.0))
      .gap_x(px(4.0))
      .child(self.input.clone())
      .child(self.counter.clone())
      .child(self.next_btn.clone())
      .child(self.prev_btn.clone())
      .child(self.close_btn.clone())
  }
}
