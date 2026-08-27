use crate::app_ext::AppExt;
use crate::db::models::{RootRoute, Route};
use gpui::*;

pub struct LibraryBtn {}

impl LibraryBtn {
  pub fn new() -> Self {
    Self {}
  }
}

impl Render for LibraryBtn {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
          let _ = cx.settings_mut().set_route(RootRoute::Main(Route::Library));
        }),
      )
      .child(svg().path("icomoon-free--books.svg").size(px(20.0)).text_color(rgb(0xD4D4D5)))
  }
}
