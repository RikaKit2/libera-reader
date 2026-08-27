use crate::ui::pages::book_viewer::constants::{RADIUS_SM, search_bar};
use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::ActiveTheme;
use gpui_component::tooltip::Tooltip;
use rust_i18n::t;

pub struct SearchCloseBtn {
  state: Entity<BookViewerState>,
}

impl SearchCloseBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl Render for SearchCloseBtn {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = cx.theme();
    let state = self.state.clone();

    div()
      .id("search-close-btn")
      .w(search_bar::CLOSE_BTN_SIZE)
      .h(search_bar::CLOSE_BTN_SIZE)
      .flex()
      .items_center()
      .justify_center()
      .rounded(RADIUS_SM)
      .cursor_pointer()
      .tooltip(|window, cx| {
        Tooltip::new(t!("components.book_viewer.tooltips.search_close").to_string())
          .build(window, cx)
      })
      .hover(move |s| s.bg(theme.foreground.opacity(0.08)))
      .on_mouse_down(
        MouseButton::Left,
        cx.listener(move |_this, _, _window, cx| {
          state.update(cx, |s, cx| {
            s.toggle_search();
            cx.notify();
          });
        }),
      )
      .child(
        svg()
          .path("material-symbols--close.svg")
          .size(search_bar::CLOSE_ICON_SIZE)
          .text_color(theme.foreground),
      )
  }
}
