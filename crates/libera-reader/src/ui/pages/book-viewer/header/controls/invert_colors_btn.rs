use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::prelude::FluentBuilder;
use gpui::*;

pub struct InvertColorsBtn {
  state: Entity<BookViewerState>,
}

impl InvertColorsBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl Render for InvertColorsBtn {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let inverted = self.state.read(cx).invert_colors;
    let state = self.state.clone();

    div()
      .w(px(28.0))
      .h(px(28.0))
      .flex()
      .items_center()
      .justify_center()
      .rounded(px(3.0))
      .cursor_pointer()
      .when(inverted, |s| s.bg(rgb(0x4A4A4F)))
      .hover(|s| s.bg(rgb(0x666667)))
      .on_mouse_down(
        MouseButton::Left,
        cx.listener(move |_this, _, _window, cx| {
          state.update(cx, |s, cx| {
            s.toggle_invert_colors();
            cx.notify();
          });
        }),
      )
      .child(
        svg()
          .path(if inverted { "heroicons--moon.svg" } else { "heroicons--sun-solid.svg" })
          .size(px(22.0))
          .text_color(rgb(0xD4D4D5)),
      )
  }
}
