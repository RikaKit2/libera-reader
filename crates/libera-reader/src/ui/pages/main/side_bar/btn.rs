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
    let hover_color = adjust_brightness(theme.foreground, 1.2);

    // The icon needs its own element id so GPUI can track hover on it directly
    // (a non-stateful svg never receives hover). The div keeps `self.id` for
    // click handling. Active icons stay bright and don't react to hover.
    let icon_id: SharedString = format!("{}_icon", self.image_source).into();
    let base_icon = svg().path(&self.image_source).id(icon_id).w(px(28.0)).h(px(28.0));

    let icon = if is_active {
      base_icon.text_color(hover_color)
    } else {
      base_icon.text_color(normal_color).hover(|h| h.text_color(hover_color))
    };

    let mut btn: Stateful<gpui::Div> = div()
      .id(self.id)
      .child(icon)
      .m_1()
      .p_2()
      .rounded_md()
      .when(is_active, |s| s.bg(theme.sidebar_accent))
      .hover(move |h| if is_active { h } else { h.bg(theme.sidebar_accent.opacity(0.5)) });

    if let Some(handler) = self.on_click {
      btn = btn.on_click(move |evt, window, cx| handler(evt, window, cx));
    }

    btn
  }
}
