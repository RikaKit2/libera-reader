use crate::ui::utils::adjust_brightness;
use gpui::prelude::*;
use gpui::{div, px, rgb, svg, App, ClickEvent, ElementId, IntoElement, ParentElement, SharedString, Styled, Window};
use libera_reader_core::ctx::GlobalCTX;
use libera_reader_core::db::models::{RootRoute, Route};

#[derive(IntoElement)]
pub(crate) struct Btn {
  id: ElementId,
  on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
  btn_route: Route,
  image_source: SharedString,
}

impl Btn {
  pub fn new(route: Route, image_source: &'static str, click_event_handler: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>) -> Self {
    Self {
      id: image_source.into(),
      on_click: click_event_handler,
      btn_route: route,
      image_source: image_source.into(),
    }
  }
  fn get_active_status(&self, cx: &mut App) -> bool {
    match cx.ctx().settings.read().route {
      RootRoute::Main(curr_route) => {
        curr_route.eq(&self.btn_route)
      }
      _ => { false }
    }
  }
}

impl RenderOnce for Btn {
  fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
    let theme = cx.ctx().settings.read().theme.data();
    let is_active = self.get_active_status(cx);
    let icon = svg().path(self.image_source).w(px(28.0)).h(px(28.0));

    let (btn, icon) = match is_active {
      true => {
        let img = icon.text_color(rgb(theme.base_color_content));
        let btn = div().border_color(rgb(theme.primary_color));
        (btn, img)
      }
      false => {
        let img = icon.text_color(rgb(adjust_brightness(theme.base_color_content, 0.6)))
          .hover(|h| h.text_color(rgb(adjust_brightness(theme.base_color_content, 0.9))));
        let btn = div().border_color(gpui::transparent_black());
        (btn, img)
      }
    };
    btn
      .id(self.id)
      .child(icon)
      .p_2()
      .border_l_2()
      .when_some(self.on_click, move |this, on_click| {
        this.on_click(move |evt, window, cx| on_click(evt, window, cx))
      })
  }
}

impl Render for Btn {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = cx.ctx().settings.read().theme.data();
    let is_active = self.get_active_status(cx);

    let icon = svg().path(self.image_source.clone())
      .w(px(28.0))
      .h(px(28.0))
      .hover(|h| h.text_color(rgb(adjust_brightness(theme.base_color_content, 0.9))));

    let (btn, icon) = match is_active {
      true => {
        let img = icon.text_color(rgb(theme.base_color_content));
        let btn = div().border_color(rgb(theme.primary_color));
        (btn, img)
      }
      false => {
        let img = icon.text_color(rgb(adjust_brightness(theme.base_color_content, 0.8)));
        let btn = div().border_color(gpui::transparent_black());
        (btn, img)
      }
    };
    btn.id(self.id.clone()).child(icon).p_2().border_l_2()
  }
}
