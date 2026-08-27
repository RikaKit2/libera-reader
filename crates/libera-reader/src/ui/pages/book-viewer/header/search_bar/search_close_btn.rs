use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;

pub struct SearchCloseBtn {
  state: Entity<BookViewerState>,
}

impl SearchCloseBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl Render for SearchCloseBtn {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let state = self.state.clone();

    div()
      .w(px(28.0))
      .h(px(28.0))
      .flex()
      .items_center()
      .justify_center()
      .rounded(px(3.0))
      .cursor_pointer()
      .hover(|s| s.bg(rgb(0x666667)))
      .on_mouse_down(
        MouseButton::Left,
        cx.listener(move |_this, _, _window, cx| {
          state.update(cx, |s, cx| {
            s.toggle_search();
            cx.notify();
          });
        }),
      )
      .child(svg().path("material-symbols--close.svg").size(px(20.0)).text_color(rgb(0xD4D4D5)))
  }
}
