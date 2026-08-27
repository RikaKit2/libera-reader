use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::ActiveTheme;

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
      "Не найдено".to_string()
    } else {
      format!("{} из {}", current, count)
    };

    div().text_sm().text_color(cx.theme().muted_foreground).px_2().child(text)
  }
}
