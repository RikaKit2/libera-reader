use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;

pub struct SearchPrevBtn {
  state: Entity<BookViewerState>,
}

impl SearchPrevBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl Render for SearchPrevBtn {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let state_ref = self.state.read(cx);
    let has_results = !state_ref.search_results.is_empty();
    let state = self.state.clone();

    div()
      .h(px(28.0))
      .px(px(6.0))
      .bg(rgb(0x4A4A4F))
      .hover(|s| s.bg(rgb(0x666667)))
      .rounded(px(3.0))
      .flex()
      .items_center()
      .gap_x(px(4.0))
      .cursor_pointer()
      .opacity(if has_results { 1.0 } else { 0.5 })
      .on_mouse_down(
        MouseButton::Left,
        cx.listener(move |_this, _, _window, cx| {
          state.update(cx, |s, cx| {
            if !s.search_results.is_empty() {
              if s.current_search_idx == 0 {
                s.current_search_idx = s.search_results.len() - 1;
              } else {
                s.current_search_idx -= 1;
              }
              let page = s.search_results[s.current_search_idx].page;
              s.go_to_page(page);
              cx.notify();
            }
          });
        }),
      )
      .child(svg().path("heroicons--chevron-up.svg").size(px(16.0)).text_color(rgb(0xD4D4D5)))
      .child(div().text_xs().text_color(rgb(0xD4D4D5)).child("Previous"))
  }
}
