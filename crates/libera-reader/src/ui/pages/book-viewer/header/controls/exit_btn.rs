use crate::app_ext::AppExt;
use crate::db::models::{RootRoute, Route};
use gpui::*;

pub struct ExitBtn {}

impl ExitBtn {
  pub fn new() -> Self {
    Self {}
  }
}

impl Render for ExitBtn {
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
      .child(svg().path("material-symbols--close.svg").size(px(22.0)).text_color(rgb(0xD4D4D5)))
  }
}
