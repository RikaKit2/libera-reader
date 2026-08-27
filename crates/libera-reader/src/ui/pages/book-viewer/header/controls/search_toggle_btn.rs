use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::tooltip::Tooltip;
use rust_i18n::t;
pub struct SearchToggleBtn {
  state: Entity<BookViewerState>,
}

impl SearchToggleBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl Render for SearchToggleBtn {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let active = self.state.read(cx).search_open;
    let state = self.state.clone();

    div()
      .id("header-search-toggle-btn")
      .w(px(28.0))
      .h(px(28.0))
      .flex()
      .items_center()
      .justify_center()
      .rounded(px(3.0))
      .cursor_pointer()
      .tooltip(|window, cx| {
        Tooltip::new(t!("components.book_viewer.tooltips.search").to_string()).build(window, cx)
      })
      .when(active, |s| s.bg(rgb(0x4A4A4F)))
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
      .child(svg().path("heroicons--magnifying-glass.svg").size(px(20.0)).text_color(rgb(0xD4D4D5)))
  }
}
