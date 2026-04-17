use gpui::{
  AbsoluteLength, App, AppContext, Context, Entity, IntoElement, ParentElement, Pixels, Render, Styled, Window,
  div, px,
};
use gpui_component::{ActiveTheme, Icon, IconName, Sizable};
use libera_reader_core::ctx::GlobalCTX;
use rust_i18n::t;

use crate::ui::{
  components::path_select::PathSelect,
  pages::setup::{back_btn, next_btn},
};

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

    let page = div().w_full().h_full().flex().flex_col().children([
      div().flex().justify_center().items_center().flex_col().children([
        div().child(Icon::new(IconName::Folder).with_size(px(40.0))),
        div()
          .mt_2()
          .flex()
          .justify_center()
          .items_center()
          .child(t!("pages.setup.pages.library.title").to_string()),
        div()
          .mt_2()
          .child(t!("pages.setup.pages.library.description").to_string())
          .text_size(AbsoluteLength::Pixels(Pixels::from(14.0))),
      ]),
      div().flex_1().children([
        div().flex().items_center().gap_1().children([
          div().child(Icon::new(IconName::FolderOpen).small()),
          div().child(t!("pages.setup.pages.library.target_dir_label").to_string()),
        ]),
        div().child(self.path_select.clone()),
      ]),
    ]);

    let path_selected = cx.ctx().settings.read().path_to_scan.is_some();
    let next_btn_disabled = !path_selected;

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
        div().w_full().flex().justify_between().children([back_btn(), next_btn(next_btn_disabled)]),
      ])
  }
}
