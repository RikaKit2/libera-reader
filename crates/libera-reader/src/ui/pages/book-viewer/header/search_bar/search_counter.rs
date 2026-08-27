use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use rust_i18n::t;

pub struct SearchCounter {
  state: Entity<BookViewerState>,
}
impl SearchCounter {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl Render for SearchCounter {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let state_ref = self.state.read(cx);
    let count = state_ref.search_results.len();
    let current = if count > 0 { state_ref.current_search_idx + 1 } else { 0 };

    let text = if state_ref.search_query.is_empty() {
      "".to_string()
    } else if count == 0 {
      t!("components.book_viewer.search.not_found").to_string()
    } else {
      t!("components.book_viewer.search.match_count", current = current, total = count).to_string()
    };
    div().text_xs().text_color(rgb(0xB3B4B7)).px(px(6.0)).whitespace_nowrap().child(text)
  }
}
