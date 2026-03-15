use gpui::{App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div};

use gpui_component::ActiveTheme;
use rust_i18n::t;

use crate::ui::pages::setup::next_btn;

pub(crate) struct Welcome {}
impl Welcome {
  pub(crate) fn new(cx: &mut App) -> Entity<Self> {
    cx.new(|_| Self {})
  }
}

impl Render for Welcome {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = cx.theme();
    let page = div().flex().flex_col().children([
      div().flex().justify_center().child(t!("pages.setup.pages.welcome.title").to_string()),
      div().mt_4().child(t!("pages.setup.pages.welcome.subtitle").to_string()),
      div().mt_2().child(t!("pages.setup.pages.welcome.features.title").to_string()).text_sm(),
      div().ml_4().flex().flex_col().justify_center().text_sm().children([
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
      .children([div().mx_1_6().child(page), div().w_full().flex().justify_end().children([next_btn(false)])])
  }
}
