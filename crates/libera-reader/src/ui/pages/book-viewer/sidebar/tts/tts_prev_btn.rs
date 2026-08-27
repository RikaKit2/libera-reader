use crate::ui::pages::book_viewer::constants::{RADIUS_SM, tts};
use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::ActiveTheme;
use gpui_component::tooltip::Tooltip;
use rust_i18n::t;

pub struct TtsPrevBtn {
  state: Entity<BookViewerState>,
}

impl TtsPrevBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl Render for TtsPrevBtn {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
      .on_mouse_down(
        MouseButton::Left,
        cx.listener(move |_this, _, _window, cx| {
          state.update(cx, |s, cx| {
            s.prev_page();
            cx.notify();
          });
        }),
      )
      .child(
        svg()
          .path("heroicons--backward-20-solid.svg")
          .size(tts::NAV_ICON_SIZE)
          .text_color(theme.foreground),
      )
  }
}
