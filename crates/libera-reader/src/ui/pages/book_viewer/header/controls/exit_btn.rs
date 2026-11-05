use crate::app_ext::AppExt;
use crate::db::models::{RootRoute, Route};
use gpui::*;
use gpui_component::{
  Icon, Sizable,
  button::{Button, ButtonVariants},
};
use rust_i18n::t;

#[derive(Default, IntoElement)]
pub struct ExitBtn {}

impl ExitBtn {
  pub fn new() -> Self {
    Self {}
  }
}

impl RenderOnce for ExitBtn {
  fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
    Button::new("header-exit-btn")
      .ghost()
      .xsmall()
      .icon(Icon::new(Icon::empty()).path("material-symbols--close.svg"))
      .tooltip(t!("components.book_viewer.tooltips.close").to_string())
      .on_click(|_, _window, cx| {
        cx.services().extraction_coordinator.set_mode(
          crate::services::extraction_coordinator::ExtractionCoordinatorMode::BackgroundLibrary,
        );
        let _ = cx.settings_mut().set_route(RootRoute::Main(Route::Library));
      })
  }
}
