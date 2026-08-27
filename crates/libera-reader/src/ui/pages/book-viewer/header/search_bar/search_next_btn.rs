use crate::ui::pages::book_viewer::constants::{RADIUS_SM, search_bar};
use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::ActiveTheme;
use gpui_component::StyledExt;
use gpui_component::tooltip::Tooltip;
use rust_i18n::t;

pub struct SearchNextBtn {
  state: Entity<BookViewerState>,
}

impl SearchNextBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl Render for SearchNextBtn {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
      .on_mouse_down(
        MouseButton::Left,
        cx.listener(move |_this, _, _window, cx| {
          state.update(cx, |s, cx| {
            if !s.search_results.is_empty() {
              s.current_search_idx = (s.current_search_idx + 1) % s.search_results.len();
              let page = s.search_results[s.current_search_idx].page;
              s.go_to_page(page);
              cx.notify();
            }
          });
        }),
      )
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
