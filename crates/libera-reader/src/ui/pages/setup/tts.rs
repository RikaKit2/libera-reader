use gpui::{App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div, px};
use gpui_component::{ActiveTheme, Icon, IconName, Sizable};
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
    let page = div().flex().flex_col().justify_center().items_center().children([
      div().child(Icon::new(IconName::Bot).with_size(px(40.0))),
      div().mt_2().flex().justify_center().items_center().child(t!("pages.setup.pages.tts.title").to_string()),
      div().mt_2().child(t!("pages.setup.pages.tts.description").to_string()).text_size(px(14.0)),
      div().mt_4().flex().flex_col().gap_2().children([
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
        div().mx_1_6().child(page),
        div().w_full().flex().justify_between().children([back_btn(), next_btn(false)]),
      ])
  }
}
