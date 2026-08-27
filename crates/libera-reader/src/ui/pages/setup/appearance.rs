use crate::ui::pages::setup::constants::{
  APPEARANCE_SELECT_WIDTH, SETUP_ICON_SIZE, SETUP_PAGE_PADDING,
};
use crate::ui::{
  components::{LangSelect, ThemeSelect},
  pages::setup::{back_btn, next_btn},
};
use gpui::{
  App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div,
};
use gpui_component::{ActiveTheme, Icon, IconName, Sizable, StyledExt};
use rust_i18n::t;

pub(crate) struct Appearance {
  lang_select: Entity<LangSelect>,
  theme_select: Entity<ThemeSelect>,
}

impl Appearance {
  pub(crate) fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
    cx.new(|cx| Self {
      lang_select: LangSelect::new(window, cx),
      theme_select: ThemeSelect::new(window, cx),
    })
  }
}

impl Render for Appearance {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = cx.theme();
    let page = div().flex().flex_col().items_center().justify_center().children([
      div().child(Icon::new(IconName::Palette).with_size(SETUP_ICON_SIZE)),
      div()
        .mt_3()
        .text_lg()
        .font_bold()
        .child(t!("pages.setup.pages.appearance.title").to_string()),
      div().mt_4().w(APPEARANCE_SELECT_WIDTH).flex().flex_col().gap_1().children([
        div().flex().items_center().gap_1().text_sm().children([
          div().child(Icon::new(IconName::Globe).small()),
          div().child(t!("pages.setup.pages.appearance.language_label").to_string()),
        ]),
        div().child(self.lang_select.clone()),
      ]),
      div().mt_3().w(APPEARANCE_SELECT_WIDTH).flex().flex_col().gap_1().children([
        div().flex().items_center().gap_1().text_sm().children([
          div().child(Icon::new(IconName::Palette).small()),
          div().child(t!("pages.setup.pages.appearance.theme_label").to_string()),
        ]),
        div().child(self.theme_select.clone()),
      ]),
      div()
        .mt_3()
        .w(APPEARANCE_SELECT_WIDTH)
        .text_center()
        .text_xs()
        .text_color(theme.foreground.opacity(0.6))
        .child(t!("pages.setup.pages.appearance.note").to_string()),
    ]);

    div()
      .bg(theme.background)
      .w_full()
      .h_full()
      .p(SETUP_PAGE_PADDING)
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
