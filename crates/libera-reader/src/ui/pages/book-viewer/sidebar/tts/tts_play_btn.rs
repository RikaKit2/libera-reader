use crate::ui::pages::book_viewer::constants::{RADIUS_MD, tts};
use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::ActiveTheme;
use gpui_component::tooltip::Tooltip;
use rust_i18n::t;

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
    let theme = cx.theme();
    let playing = self.state.read(cx).tts_playing;
    let state = self.state.clone();

    div()
      .id("tts-play-btn")
      .w(tts::PLAY_BTN_SIZE)
      .h(tts::PLAY_BTN_SIZE)
      .flex()
      .items_center()
      .justify_center()
      .rounded(RADIUS_MD)
      .bg(theme.primary.opacity(0.2))
      .hover(move |s| s.bg(theme.primary.opacity(0.35)))
      .tooltip(|window, cx| {
        Tooltip::new(t!("components.book_viewer.tooltips.tts_play").to_string()).build(window, cx)
      })
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
          .path(if playing { "ri--pause-fill.svg" } else { "ri--play-fill.svg" })
          .size(tts::PLAY_ICON_SIZE)
          .text_color(theme.primary),
      )
  }
}
