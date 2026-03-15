use gpui::{
  AbsoluteLength, App, AppContext, Context, Entity, IntoElement, ParentElement, Pixels, Render, Styled, Window,
  div, px,
};

use gpui_component::{ActiveTheme, Icon, IconName, Sizable};
use rust_i18n::t;

use crate::ui::{
  components::{LangSelect, ThemeSelect},
  pages::setup::{back_btn, next_btn},
};

pub(crate) struct Appearance {
  lang_select: Entity<LangSelect>,
  theme_select: Entity<ThemeSelect>,
}

impl Appearance {
  pub(crate) fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
    cx.new(|cx| Self { lang_select: LangSelect::new(window, cx), theme_select: ThemeSelect::new(window, cx) })
  }
}

impl Render for Appearance {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = cx.theme();
    let page = div().children([div().flex().flex_col().justify_center().items_center().children([
      div().child(Icon::new(IconName::Palette).with_size(px(40.0))),
      div()
        .mt_2()
        .flex()
        .justify_center()
        .items_center()
        .child(t!("pages.setup.pages.appearance.title").to_string()),
      div().w(px(210.0)).flex().items_start().justify_start().items_center().gap_1().children([
        div().child(Icon::new(IconName::Globe).small()),
        div().child(t!("pages.setup.pages.appearance.language_label").to_string()),
      ]),
      div().my_2().child(self.lang_select.clone()),
      div().w(px(210.0)).flex().justify_start().items_center().gap_1().children([
        div().child(Icon::new(IconName::Palette).small()),
        div().child(t!("pages.setup.pages.appearance.theme_label").to_string()),
      ]),
      div().my_2().child(self.theme_select.clone()),
      div()
        .w(px(210.0))
        .flex()
        .items_center()
        .justify_start()
        .child(t!("pages.setup.pages.appearance.note").to_string())
        .text_size(AbsoluteLength::Pixels(Pixels::from(12.0))),
    ])]);

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
