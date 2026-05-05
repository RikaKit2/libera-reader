use gpui::{
  App, ClickEvent, ElementId, IntoElement, ParentElement, SharedString, Styled, Window, div, px,
  svg,
};
use gpui::{Stateful, prelude::*};
use gpui_component::ActiveTheme;
use libera_reader_core::ctx::GlobalCTX;
use libera_reader_core::db::models::{RootRoute, Route};

use crate::app_utils::adjust_brightness;

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub(crate) struct Btn {
  id: ElementId,
  on_click: Option<ClickHandler>,
  btn_route: Route,
  image_source: SharedString,
}

impl Btn {
  pub fn new(
    route: Route, image_source: &'static str, click_event_handler: Option<ClickHandler>,
  ) -> Self {
    Self {
      id: image_source.into(),
      on_click: click_event_handler,
      btn_route: route,
      image_source: image_source.into(),
    }
  }

  fn get_active_status(&self, cx: &mut App) -> bool {
    match cx.ctx().settings.read().route {
      RootRoute::Main(curr_route) => curr_route.eq(&self.btn_route),
      _ => false,
    }
  }

  fn build_icon_and_button(&self, is_active: bool, cx: &mut App) -> (gpui::Div, gpui::Svg) {
    let base_icon = svg().path(&self.image_source).w(px(28.0)).h(px(28.0));

    match is_active {
      true => {
        let img_text_color = adjust_brightness(cx.theme().foreground, 1.2);

        let icon = base_icon.text_color(img_text_color);
        let btn = div().border_color(cx.theme().primary);
        (btn, icon)
      }
      false => {
        let img_text_color = adjust_brightness(cx.theme().foreground, 1.0);
        let img_hover_color = adjust_brightness(cx.theme().foreground, 1.2);

        let icon = base_icon.text_color(img_text_color).hover(|h| h.text_color(img_hover_color));
        let btn = div().border_color(gpui::transparent_black());
        (btn, icon)
      }
    }
  }
}

impl RenderOnce for Btn {
  fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
    let is_active = self.get_active_status(cx);
    let (btn, icon) = self.build_icon_and_button(is_active, cx);

    fn attach_click_handler(
      this: Stateful<gpui::Div>, on_click: ClickHandler,
    ) -> Stateful<gpui::Div> {
      this.on_click(move |evt, window, cx| on_click(evt, window, cx))
    }

    btn.id(self.id).child(icon).p_2().border_l_2().when_some(self.on_click, attach_click_handler)
  }
}

impl Render for Btn {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let is_active = self.get_active_status(cx);
    let (btn, icon) = self.build_icon_and_button(is_active, cx);

    btn.id(self.id.clone()).child(icon).p_2().border_l_2()
  }
}
