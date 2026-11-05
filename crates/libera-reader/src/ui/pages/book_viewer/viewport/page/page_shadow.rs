use gpui::*;
use gpui_component::ActiveTheme;

/// Page drop shadow and subtle border overlay.
#[derive(IntoElement, Default)]
pub struct PageShadow {}

impl PageShadow {
  pub fn new() -> Self {
    Self {}
  }
}

impl RenderOnce for PageShadow {
  fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
    div().absolute().inset_0().rounded_sm().shadow_lg().border_1().border_color(cx.theme().border)
  }
}
