use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::prelude::FluentBuilder;
use gpui::*;

pub struct FullscreenBtn {
  state: Entity<BookViewerState>,
}

impl FullscreenBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl Render for FullscreenBtn {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let is_fullscreen = self.state.read(cx).is_fullscreen;
    let state = self.state.clone();

    div()
      .w(px(28.0))
      .h(px(28.0))
      .flex()
      .items_center()
      .justify_center()
      .rounded(px(3.0))
      .cursor_pointer()
      .when(is_fullscreen, |s| s.bg(rgb(0x4A4A4F)))
      .hover(|s| s.bg(rgb(0x666667)))
      .on_mouse_down(
        MouseButton::Left,
        cx.listener(move |_this, _, window, cx| {
          state.update(cx, |s, cx| {
            s.is_fullscreen = !s.is_fullscreen;
            window.toggle_fullscreen();
            cx.notify();
          });
        }),
      )
      .child(
        svg()
          .path(if is_fullscreen {
            "gridicons--fullscreen-exit.svg"
          } else {
            "gridicons--fullscreen.svg"
          })
          .size(px(20.0))
          .text_color(rgb(0xD4D4D5)),
      )
  }
}
