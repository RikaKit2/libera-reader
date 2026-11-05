use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::ActiveTheme;
use rust_i18n::t;

#[derive(IntoElement)]
pub struct PageCounter {
  state: Entity<BookViewerState>,
}

impl PageCounter {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl RenderOnce for PageCounter {
  fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
    let theme = cx.theme();
    let state_ref = self.state.read(cx);
    let current = state_ref.current_page;
    let total = state_ref.total_pages;

    div().text_sm().text_color(theme.foreground).child(
      t!("components.book_viewer.pagination.page_of", current = current, total = total).to_string(),
    )
  }
}
