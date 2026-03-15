use gpui::{App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div, px};
use gpui_component::{ActiveTheme, Icon, Sizable};
use rust_i18n::t;

use crate::ui::pages::setup::{back_btn, next_btn};

pub(crate) struct SyncPage {}
impl SyncPage {
  pub(crate) fn new(cx: &mut App) -> Entity<Self> {
    cx.new(|_| Self {})
  }
}

impl Render for SyncPage {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = cx.theme();
    let page = div().flex().flex_col().justify_center().items_center().children([
      div().child(Icon::new(Icon::empty()).path("refresh-ccw.svg").with_size(px(40.0))),
      div().mt_2().flex().justify_center().items_center().child(t!("pages.setup.pages.sync.title").to_string()),
      div().mt_2().child(t!("pages.setup.pages.sync.description").to_string()).text_size(px(14.0)),
    ]);

    div()
      .bg(theme.background)
      .w_full()
      .h_full()
      .p_6()
      .flex()
      .flex_col()
      .justify_between()
      .text_color(theme.foreground)
      .children([
        div().mx_1_6().child(page),
        div().w_full().flex().justify_between().children([back_btn(), next_btn(false)]),
      ])
  }
}
