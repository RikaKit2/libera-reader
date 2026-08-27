use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;

pub struct TtsPlayBtn {
  state: Entity<BookViewerState>,
}

impl TtsPlayBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl Render for TtsPlayBtn {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let playing = self.state.read(cx).tts_playing;
    let state = self.state.clone();

    div()
      .w(px(32.0))
      .h(px(32.0))
      .flex()
      .items_center()
      .justify_center()
      .rounded(px(4.0))
      .bg(rgb(0x4A4A4F))
      .hover(|s| s.bg(rgb(0x666667)))
      .cursor_pointer()
      .on_mouse_down(
        MouseButton::Left,
        cx.listener(move |_this, _, _window, cx| {
          state.update(cx, |s, cx| {
            s.tts_playing = !s.tts_playing;
            cx.notify();
          });
        }),
      )
      .child(
        svg()
          .path(if playing { "heroicons--moon.svg" } else { "ri--play-fill.svg" })
          .size(px(20.0))
          .text_color(rgb(0xD4D4D5)),
      )
  }
}
