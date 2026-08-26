use gpui::{
  App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div, px,
};
use gpui_component::{ActiveTheme, Icon, IconName, Sizable, StyledExt};
use rust_i18n::t;

use crate::ui::pages::setup::{back_btn, next_btn};

pub(crate) struct TTSPage {}
impl TTSPage {
  pub(crate) fn new(cx: &mut App) -> Entity<Self> {
    cx.new(|_| Self {})
  }
}

impl Render for TTSPage {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = cx.theme();
    let page = div().flex().flex_col().items_center().justify_center().max_w(px(480.0)).children([
      div().child(Icon::new(IconName::Bot).with_size(px(48.0))),
      div().mt_3().text_lg().font_semibold().child(t!("pages.setup.pages.tts.title").to_string()),
      div()
        .mt_2()
        .text_sm()
        .text_center()
        .text_color(theme.foreground.opacity(0.8))
        .child(t!("pages.setup.pages.tts.description").to_string()),
      div().mt_4().flex().flex_col().gap_2().text_sm().children([
        div().child(t!("pages.setup.pages.tts.providers.offline").to_string()),
        div().child(t!("pages.setup.pages.tts.providers.online").to_string()),
      ]),
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
        div().flex_1().flex().flex_col().items_center().justify_center().child(page),
        div().w_full().flex().justify_between().children([back_btn(), next_btn(false)]),
      ])
  }
}
