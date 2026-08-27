use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;

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
    let state = self.state.clone();

    div()
      .w_full()
      .py(px(4.0))
      .px(px(8.0))
      .bg(rgb(0x4A4A4F))
      .hover(|s| s.bg(rgb(0x666667)))
      .rounded(px(3.0))
      .flex()
      .items_center()
      .justify_center()
      .cursor_pointer()
      .text_xs()
      .text_color(rgb(0xD4D4D5))
      .child("Вернуть значения по умолчанию")
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
