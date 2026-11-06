use crate::app_ext::AppExt;
use crate::db::models::{RootRoute, Route};
use crate::ui::pages::book_viewer::constants::{RADIUS_SM, header};
use gpui::*;
use gpui_component::{ActiveTheme, tooltip::Tooltip};
use rust_i18n::t;

#[derive(Default, IntoElement)]
pub struct ExitBtn {}

impl ExitBtn {
  pub fn new() -> Self {
    Self {}
  }
}

impl RenderOnce for ExitBtn {
  fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
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
      .on_click(|_, _window, cx| {
        cx.services().extraction_coordinator.set_mode(
          crate::services::extraction_coordinator::ExtractionCoordinatorMode::BackgroundLibrary,
        );
        let _ = cx.settings_mut().set_route(RootRoute::Main(Route::Library));
      })
      .child(
        svg()
          .path("material-symbols--close.svg")
          .size(header::ICON_SIZE_LG)
          .text_color(theme.foreground),
      )
  }
}
