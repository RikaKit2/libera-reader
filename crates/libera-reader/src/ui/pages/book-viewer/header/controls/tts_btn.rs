use crate::ui::pages::book_viewer::state::{BookViewerState, SidebarTab};
use gpui::prelude::FluentBuilder;
use gpui::*;

pub struct TtsBtn {
  state: Entity<BookViewerState>,
}

impl TtsBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl Render for TtsBtn {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let active = self.state.read(cx).active_sidebar_tab == SidebarTab::Tts;
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
            s.toggle_sidebar_tab(SidebarTab::Tts);
            cx.notify();
          });
        }),
      )
      .child(svg().path("ri--play-fill.svg").size(px(18.0)).text_color(rgb(0xD4D4D5)))
  }
}
