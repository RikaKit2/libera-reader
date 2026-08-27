use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::tooltip::Tooltip;
use rust_i18n::t;
pub struct PrevPageBtn {
  state: Entity<BookViewerState>,
}

impl PrevPageBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl Render for PrevPageBtn {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let state_ref = self.state.read(cx);
    let can_go_prev = state_ref.current_page > 1;
    let state = self.state.clone();

    div()
      .id("header-prev-page-btn")
      .w(px(28.0))
      .h(px(28.0))
      .flex()
      .items_center()
      .justify_center()
      .rounded(px(3.0))
      .cursor_pointer()
      .tooltip(|window, cx| {
        Tooltip::new(t!("components.book_viewer.tooltips.prev_page").to_string()).build(window, cx)
      })
      .hover(|s| s.bg(rgb(0x666667)))
      .on_mouse_down(
        MouseButton::Left,
        cx.listener(move |_this, _, _window, cx| {
          state.update(cx, |s, cx| {
            s.prev_page();
            cx.notify();
          });
        }),
      )
      .child(svg().path("heroicons--chevron-up.svg").size(px(18.0)).text_color(if can_go_prev {
        rgb(0xD4D4D5)
      } else {
        rgb(0x808085)
      }))
  }
}
