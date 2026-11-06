use crate::ui::pages::book_viewer::constants::{RADIUS_SM, search_bar};
use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::{ActiveTheme, StyledExt, tooltip::Tooltip};
use rust_i18n::t;

#[derive(IntoElement)]
pub struct SearchNextBtn {
  state: Entity<BookViewerState>,
}

impl SearchNextBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl RenderOnce for SearchNextBtn {
  fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
    let theme = cx.theme();
    let state = self.state.clone();

    div()
      .id("search-next-btn")
      .h(search_bar::BTN_HEIGHT)
      .px(search_bar::BTN_PADDING_X)
      .hover(move |s| s.bg(theme.foreground.opacity(0.12)))
      .rounded(RADIUS_SM)
      .tooltip(|window, cx| {
        Tooltip::new(t!("components.book_viewer.tooltips.search_next").to_string())
          .build(window, cx)
      })
      .flex()
      .items_center()
      .justify_center()
      .gap_x(search_bar::BTN_GAP_X)
      .cursor_pointer()
      .on_click(move |_, _window, cx| {
        super::execute_search(state.clone(), cx);
      })
      .child(
        svg()
          .path("heroicons--chevron-down.svg")
          .size(search_bar::BTN_ICON_SIZE)
          .text_color(theme.foreground),
      )
      .child(
        div()
          .text_xs()
          .font_medium()
          .text_color(theme.foreground)
          .child(t!("components.book_viewer.search.next_btn").to_string()),
      )
  }
}
