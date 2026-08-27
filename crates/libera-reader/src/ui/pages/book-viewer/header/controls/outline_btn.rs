use crate::ui::pages::book_viewer::state::{BookViewerState, SidebarTab};
use gpui::prelude::FluentBuilder;
use gpui::*;

pub struct OutlineBtn {
  state: Entity<BookViewerState>,
}

impl OutlineBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl Render for OutlineBtn {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let active = self.state.read(cx).active_sidebar_tab == SidebarTab::Outline;
    let state = self.state.clone();

    div()
      .w(px(28.0))
      .h(px(28.0))
      .flex()
      .items_center()
      .justify_center()
      .rounded(px(3.0))
      .cursor_pointer()
      .when(active, |s| s.bg(rgb(0x4A4A4F)))
      .hover(|s| s.bg(rgb(0x666667)))
      .on_mouse_down(
        MouseButton::Left,
        cx.listener(move |_this, _, _window, cx| {
          state.update(cx, |s, cx| {
            s.toggle_sidebar_tab(SidebarTab::Outline);
            cx.notify();
          });
        }),
      )
      .child(svg().path("material-symbols--list.svg").size(px(22.0)).text_color(rgb(0xD4D4D5)))
  }
}
