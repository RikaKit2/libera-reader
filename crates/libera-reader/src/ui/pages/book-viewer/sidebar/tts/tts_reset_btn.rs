use crate::ui::pages::book_viewer::constants::{RADIUS_SM, tts};
use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::ActiveTheme;
use rust_i18n::t;

pub struct TtsResetBtn {
  state: Entity<BookViewerState>,
}

impl TtsResetBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl Render for TtsResetBtn {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = cx.theme();
    let state = self.state.clone();

    div()
      .id("tts-reset-btn")
      .w_full()
      .py(tts::RESET_BTN_PY)
      .px(tts::RESET_BTN_PX)
      .bg(theme.foreground.opacity(0.06))
      .hover(move |s| s.bg(theme.foreground.opacity(0.12)))
      .rounded(RADIUS_SM)
      .flex()
      .items_center()
      .justify_center()
      .cursor_pointer()
      .text_xs()
      .text_color(theme.foreground)
      .child(t!("components.book_viewer.tts.reset_btn").to_string())
      .on_mouse_down(
        MouseButton::Left,
        cx.listener(move |_this, _, _window, cx| {
          state.update(cx, |s, cx| {
            s.tts_speed = 1.0;
            s.tts_pause_ms = 150;
            s.tts_auto_turn = true;
            cx.notify();
          });
        }),
      )
  }
}
