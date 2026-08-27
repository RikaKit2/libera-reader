use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::{ActiveTheme, checkbox::*};
use rust_i18n::t;
pub struct TtsAutoTurnToggle {
  state: Entity<BookViewerState>,
}

impl TtsAutoTurnToggle {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl Render for TtsAutoTurnToggle {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let auto_turn = self.state.read(cx).tts_auto_turn;
    let state = self.state.clone();

    div()
      .w_full()
      .flex()
      .items_center()
      .gap_x_2()
      .cursor_pointer()
      .on_mouse_down(
        MouseButton::Left,
        cx.listener(move |_this, _, _window, cx| {
          state.update(cx, |s, cx| {
            s.tts_auto_turn = !s.tts_auto_turn;
            cx.notify();
          });
        }),
      )
      .child(Checkbox::new("tts-auto-turn-check").checked(auto_turn))
      .child(
        div()
          .text_xs()
          .text_color(cx.theme().foreground)
          .child(t!("components.book_viewer.tts.auto_turn_label").to_string()),
      )
  }
}
