use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::StyledExt;
use gpui_component::tooltip::Tooltip;
use rust_i18n::t;
pub struct SearchNextBtn {
  state: Entity<BookViewerState>,
}

impl SearchNextBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl Render for SearchNextBtn {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let state_ref = self.state.read(cx);
    let has_results = !state_ref.search_results.is_empty();
    let state = self.state.clone();

    div()
      .id("search-next-btn")
      .h(px(26.0))
      .px(px(8.0))
      .bg(rgb(0x4A4A4F))
      .hover(|s| s.bg(rgb(0x666667)))
      .rounded(px(3.0))
      .tooltip(|window, cx| {
        Tooltip::new(t!("components.book_viewer.tooltips.search_next").to_string())
          .build(window, cx)
      })
      .flex()
      .items_center()
      .gap_x(px(4.0))
      .cursor_pointer()
      .on_mouse_down(
        MouseButton::Left,
        cx.listener(move |_this, _, _window, cx| {
          state.update(cx, |s, cx| {
            if !s.search_results.is_empty() {
              s.current_search_idx = (s.current_search_idx + 1) % s.search_results.len();
              let page = s.search_results[s.current_search_idx].page;
              s.go_to_page(page);
              cx.notify();
            }
          });
        }),
      )
      .child(svg().path("heroicons--chevron-down.svg").size(px(16.0)).text_color(if has_results {
        rgb(0xD4D4D5)
      } else {
        rgb(0x909095)
      }))
      .child(
        div()
          .text_xs()
          .font_medium()
          .text_color(if has_results { rgb(0xD4D4D5) } else { rgb(0x909095) })
          .child(t!("components.book_viewer.search.next_btn").to_string()),
      )
  }
}
