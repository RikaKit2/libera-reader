use crate::ui::utils::adjust_brightness;
use gpui::{App, ClickEvent, ElementId, IntoElement, ParentElement, SharedString, Styled, Window, div, px, svg};
use gpui::{Stateful, prelude::*};
use gpui_component::ActiveTheme;
use libera_reader_core::ctx::GlobalCTX;
use libera_reader_core::db::models::{RootRoute, Route};

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub(crate) struct Btn {
  id: ElementId,
  on_click: Option<ClickHandler>,
  btn_route: Route,
  image_source: SharedString,
}

impl Btn {
  pub fn new(route: Route, image_source: &'static str, click_event_handler: Option<ClickHandler>) -> Self {
    Self { id: image_source.into(), on_click: click_event_handler, btn_route: route, image_source: image_source.into() }
  }
  fn get_active_status(&self, cx: &mut App) -> bool {
    match cx.ctx().settings.read().route {
      RootRoute::Main(curr_route) => curr_route.eq(&self.btn_route),
      _ => false,
    }
  }
}

impl RenderOnce for Btn {
  fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
    let is_active = &self.get_active_status(cx);
    let icon = svg().path(self.image_source).w(px(28.0)).h(px(28.0));

    let (btn, icon) = match is_active {
      true => {
        let img = icon.text_color(cx.theme().foreground);
        let btn = div().border_color(cx.theme().primary);
        (btn, img)
      }
      false => {
        let img_text_color = adjust_brightness(cx.theme().foreground, 0.6);
        let img_hover_color = adjust_brightness(cx.theme().foreground, 0.9);

        let img = icon.text_color(img_text_color).hover(|h| h.text_color(img_hover_color));
        let btn = div().border_color(gpui::transparent_black());
        (btn, img)
      }
    };
    fn fun_name(this: Stateful<gpui::Div>, on_click: ClickHandler) -> Stateful<gpui::Div> {
      this.on_click(move |evt, window, cx| on_click(evt, window, cx))
    }
    btn.id(self.id).child(icon).p_2().border_l_2().when_some(self.on_click, fun_name)
  }
}

impl Render for Btn {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let is_active = self.get_active_status(cx);

    let icon_hover_color = adjust_brightness(cx.theme().foreground, 0.9);

    #[rustfmt::skip]
    let icon = svg()
      .path(self.image_source.clone())
      .w(px(28.0))
      .h(px(28.0))
      .hover(|h| h.text_color(icon_hover_color));

    let (btn, icon) = match is_active {
      true => {
        let img = icon.text_color(cx.theme().foreground);
        let btn = div().border_color(cx.theme().primary);
        (btn, img)
      }
      false => {
        let img = icon.text_color(adjust_brightness(cx.theme().foreground, 0.8));
        let btn = div().border_color(gpui::transparent_black());
        (btn, img)
      }
    };
    btn.id(self.id.clone()).child(icon).p_2().border_l_2()
  }
}
