use crate::app_ext::AppExt;
use crate::db::models::{RootRoute, Route};
use crate::ui::pages::book_viewer::constants::{RADIUS_SM, header};
use gpui::*;
use gpui_component::ActiveTheme;
use gpui_component::tooltip::Tooltip;
use rust_i18n::t;

pub struct ExitBtn {}

impl ExitBtn {
  pub fn new() -> Self {
    Self {}
  }
}

impl Render for ExitBtn {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = cx.theme();

    div()
      .id("header-exit-btn")
      .w(header::BTN_SIZE)
      .h(header::BTN_SIZE)
      .flex()
      .items_center()
      .justify_center()
      .rounded(RADIUS_SM)
      .cursor_pointer()
      .tooltip(|window, cx| {
        Tooltip::new(t!("components.book_viewer.tooltips.close").to_string()).build(window, cx)
      })
      .hover(move |s| s.bg(theme.foreground.opacity(0.08)))
      .on_mouse_down(
        MouseButton::Left,
        cx.listener(move |_this, _, _window, cx| {
          let _ = cx.settings_mut().set_route(RootRoute::Main(Route::Library));
        }),
      )
      .child(
        svg()
          .path("material-symbols--close.svg")
          .size(header::ICON_SIZE_LG)
          .text_color(theme.foreground),
      )
  }
}
