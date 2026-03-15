use crate::ui::components::theme_select::ThemeSelect;

use gpui::{
  App, AppContext, Context, Entity, FontWeight, IntoElement, ParentElement, Render, Styled, Window, div,
};
use gpui_component::checkbox::Checkbox;
use libera_reader_core::ctx::GlobalCTX;
use rust_i18n::t;

pub(crate) struct Settings {
  theme_select: Entity<ThemeSelect>,
}

impl Settings {
  pub(crate) fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
    cx.new(|c| Self { theme_select: ThemeSelect::new(window, c) })
  }
}

impl Render for Settings {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let guard = cx.ctx().settings.read();
    let pdf_status = guard.pdf;
    let epub_status = guard.epub;
    let mobi_status = guard.mobi;
    div().flex().flex_col().mt_2().pl_1_4().gap_y_2().children([
      div().font_weight(FontWeight::MEDIUM).text_xl().child(t!("pages.settings.used_formats").to_string()),
      div().ml_5().children([
        div().flex().gap_2().items_center().children([
          Checkbox::new("pdf_checkbox").label("pdf").checked(pdf_status).on_click(|_, _, cx| {
            cx.ctx_mut().settings.invert_pdf().unwrap();
          }),
          Checkbox::new("epub_checkbox").label("epub").checked(epub_status).on_click(|_, _, cx| {
            cx.ctx_mut().settings.invert_epub().unwrap();
          }),
          Checkbox::new("mobi_checkbox").label("mobi").checked(mobi_status).on_click(|_, _, cx| {
            cx.ctx_mut().settings.invert_mobi().unwrap();
          }),
        ]),
        div(),
      ]),
      div().child(t!("pages.settings.theme").to_string()),
      div().child(self.theme_select.clone()),
    ])
  }
}
