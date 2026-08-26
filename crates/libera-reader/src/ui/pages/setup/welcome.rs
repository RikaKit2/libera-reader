use crate::ui::pages::setup::next_btn;
use gpui::{
  App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div, px,
};
use gpui_component::{ActiveTheme, StyledExt};
use rust_i18n::t;

pub(crate) struct Welcome {}
impl Welcome {
  pub(crate) fn new(cx: &mut App) -> Entity<Self> {
    cx.new(|_| Self {})
  }
}

impl Render for Welcome {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = cx.theme();
    let page = div().flex().flex_col().items_center().justify_center().max_w(px(520.0)).children([
      div().text_xl().font_bold().child(t!("pages.setup.pages.welcome.title").to_string()),
      div()
        .mt_3()
        .text_center()
        .text_sm()
        .text_color(theme.foreground.opacity(0.8))
        .child(t!("pages.setup.pages.welcome.subtitle").to_string()),
      div()
        .mt_5()
        .w_full()
        .p_4()
        .rounded_lg()
        .bg(theme.foreground.opacity(0.04))
        .border_1()
        .border_color(theme.foreground.opacity(0.08))
        .flex()
        .flex_col()
        .gap_2()
        .text_sm()
        .children([
          div().font_bold().child(t!("pages.setup.pages.welcome.features.title").to_string()),
          div().child(t!("pages.setup.pages.welcome.features.tts").to_string()),
          div().child(t!("pages.setup.pages.welcome.features.fs").to_string()),
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
        div().w_full().flex().justify_end().children([next_btn(false)]),
      ])
  }
}
