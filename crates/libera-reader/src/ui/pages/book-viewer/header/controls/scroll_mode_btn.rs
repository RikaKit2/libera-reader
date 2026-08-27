use crate::ui::pages::book_viewer::constants::{RADIUS_SM, header};
use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::ActiveTheme;
use gpui_component::tooltip::Tooltip;
use rust_i18n::t;

pub struct ScrollModeBtn {
  _state: Entity<BookViewerState>,
}

impl ScrollModeBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { _state: state }
  }
}

impl Render for ScrollModeBtn {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = cx.theme();

    div()
      .id("header-scroll-mode-btn")
      .w(header::BTN_SIZE)
      .h(header::BTN_SIZE)
      .flex()
      .items_center()
      .justify_center()
      .rounded(RADIUS_SM)
      .cursor_pointer()
      .tooltip(|window, cx| {
        Tooltip::new(t!("components.book_viewer.tooltips.scroll_mode").to_string())
          .build(window, cx)
      })
      .hover(move |s| s.bg(theme.foreground.opacity(0.08)))
      .child(
        svg()
          .path("pajamas--scroll-down.svg")
          .size(header::ICON_SIZE_SM)
          .text_color(theme.foreground),
      )
  }
}
