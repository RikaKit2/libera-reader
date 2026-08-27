use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;

pub struct NextPageBtn {
  state: Entity<BookViewerState>,
}

impl NextPageBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl Render for NextPageBtn {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let state_ref = self.state.read(cx);
    let can_go_next = state_ref.current_page < state_ref.total_pages;
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
      .opacity(if can_go_next { 1.0 } else { 0.4 })
      .on_mouse_down(
        MouseButton::Left,
        cx.listener(move |_this, _, _window, cx| {
          state.update(cx, |s, cx| {
            s.next_page();
            cx.notify();
          });
        }),
      )
      .child(svg().path("heroicons--chevron-down.svg").size(px(18.0)).text_color(rgb(0xD4D4D5)))
  }
}
