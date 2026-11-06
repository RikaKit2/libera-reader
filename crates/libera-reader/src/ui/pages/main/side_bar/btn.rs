use crate::app_ext::AppExt;
use crate::db::models::{RootRoute, Route};
use gpui::{
  App, ClickEvent, ElementId, IntoElement, ParentElement, SharedString, Styled, Window, div, px,
  svg,
};
use gpui::{Stateful, prelude::*};
use gpui_component::ActiveTheme;

use crate::settings::adjust_brightness;

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
    match cx.settings().read().route {
      RootRoute::Main(curr_route) => curr_route.eq(&self.btn_route),
      _ => false,
    }
  }
}

impl RenderOnce for Btn {
  fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
    let is_active = self.get_active_status(cx);
    let theme = cx.theme();

    let normal_color = adjust_brightness(theme.foreground, 1.0);
    let hover_color = adjust_brightness(theme.foreground, 1.25);

    let icon = svg().path(&self.image_source).w(px(28.0)).h(px(28.0)).text_color(if is_active {
      theme.primary
    } else {
      normal_color
    });

    let mut btn: Stateful<gpui::Div> = div()
      .id(self.id)
      .w_12()
      .h_12()
      .flex()
      .items_center()
      .justify_center()
      .cursor_pointer()
      .border_l_2()
      .border_color(if is_active { theme.primary } else { gpui::transparent_black() })
      .when(is_active, |s| s.bg(theme.primary.opacity(0.12)))
      .hover(move |s| s.bg(theme.foreground.opacity(0.08)).text_color(hover_color))
      .child(icon);

    if let Some(handler) = self.on_click {
      btn = btn.on_click(move |evt, window, cx| handler(evt, window, cx));
    }

    btn
  }
}
