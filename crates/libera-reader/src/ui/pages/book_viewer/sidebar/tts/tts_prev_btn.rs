use crate::ui::pages::book_viewer::constants::{RADIUS_SM, tts};
use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::{ActiveTheme, tooltip::Tooltip};
use rust_i18n::t;

#[derive(IntoElement)]
pub struct TtsPrevBtn {
  state: Entity<BookViewerState>,
}

impl TtsPrevBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl RenderOnce for TtsPrevBtn {
  fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
    let theme = cx.theme();
    let state = self.state.clone();

    div()
      .id("tts-prev-btn")
      .w(tts::NAV_BTN_SIZE)
      .h(tts::NAV_BTN_SIZE)
      .flex()
      .items_center()
      .justify_center()
      .rounded(RADIUS_SM)
      .hover(move |s| s.bg(theme.foreground.opacity(0.08)))
      .tooltip(|window, cx| {
        Tooltip::new(t!("components.book_viewer.tooltips.tts_prev").to_string()).build(window, cx)
      })
      .cursor_pointer()
      .on_click(move |_, _window, cx| {
        state.update(cx, |s, cx| {
          s.prev_page();
          cx.notify();
        });
      })
      .child(
        svg()
          .path("heroicons--backward-20-solid.svg")
          .size(tts::NAV_ICON_SIZE)
          .text_color(theme.foreground),
      )
  }
}
