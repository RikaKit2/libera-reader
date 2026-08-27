use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::tooltip::Tooltip;
use rust_i18n::t;
pub struct ScrollModeBtn {
  state: Entity<BookViewerState>,
}

impl ScrollModeBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl Render for ScrollModeBtn {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let continuous = self.state.read(cx).layout_mode.is_continuous();
    let state = self.state.clone();

    div()
      .id("header-scroll-mode-btn")
      .w(px(28.0))
      .h(px(28.0))
      .flex()
      .items_center()
      .justify_center()
      .rounded(px(3.0))
      .cursor_pointer()
      .tooltip(|window, cx| {
        Tooltip::new(t!("components.book_viewer.tooltips.scroll_mode").to_string())
          .build(window, cx)
      })
      .when(continuous, |s| s.bg(rgb(0x4A4A4F)))
      .hover(|s| s.bg(rgb(0x666667)))
      .on_mouse_down(
        MouseButton::Left,
        cx.listener(move |_this, _, _window, cx| {
          state.update(cx, |s, cx| {
            s.layout_mode.toggle_continuous();
            cx.notify();
          });
        }),
      )
      .child(svg().path("pajamas--scroll-down.svg").size(px(18.0)).text_color(rgb(0xD4D4D5)))
  }
}
