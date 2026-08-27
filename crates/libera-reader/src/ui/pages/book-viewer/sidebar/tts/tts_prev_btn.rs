use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
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
    let state = self.state.clone();

    div()
      .id("tts-prev-btn")
      .w(px(28.0))
      .h(px(28.0))
      .flex()
      .items_center()
      .justify_center()
      .rounded(px(3.0))
      .hover(|s| s.bg(rgb(0x666667)))
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
        svg().path("heroicons--backward-20-solid.svg").size(px(20.0)).text_color(rgb(0xD4D4D5)),
      )
  }
}
