use gpui::*;
use gpui_component::ActiveTheme;

pub struct PageShadow {}

impl PageShadow {
  pub fn new() -> Self {
    Self {}
  }
}

impl Render for PageShadow {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    div().absolute().inset_0().rounded_sm().shadow_lg().border_1().border_color(cx.theme().border)
  }
}
