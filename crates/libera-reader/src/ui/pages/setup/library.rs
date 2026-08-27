use crate::app_ext::AppExt;
use crate::ui::pages::setup::constants::{MAX_CONTENT_WIDTH, SETUP_ICON_SIZE, SETUP_PAGE_PADDING};
use crate::ui::{
  components::path_select::PathSelect,
  pages::setup::{back_btn, next_btn},
};
use gpui::{
  App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div,
};
use gpui_component::{ActiveTheme, Icon, IconName, Sizable, StyledExt};
use rust_i18n::t;

pub(crate) struct Library {
  path_select: Entity<PathSelect>,
}

impl Library {
  pub(crate) fn new(cx: &mut App) -> Entity<Self> {
    cx.new(|cx| Self { path_select: PathSelect::new(cx) })
  }
}

impl Render for Library {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = cx.theme();

    let page =
      div().flex().flex_col().items_center().justify_center().max_w(MAX_CONTENT_WIDTH).children([
        div().child(Icon::new(IconName::Folder).with_size(SETUP_ICON_SIZE)),
        div()
          .mt_3()
          .text_lg()
          .font_semibold()
          .child(t!("pages.setup.pages.library.title").to_string()),
        div()
          .mt_1()
          .text_sm()
          .text_center()
          .text_color(theme.foreground.opacity(0.7))
          .child(t!("pages.setup.pages.library.description").to_string()),
        div().mt_6().child(self.path_select.clone()),
      ]);
    let path_selected = cx.settings().read().path_to_scan.is_some();
    let next_btn_disabled = !path_selected;

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
        div().w_full().flex().justify_between().children([back_btn(), next_btn(next_btn_disabled)]),
      ])
  }
}
