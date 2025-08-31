use crate::ui::components::checkbox::Checkbox;
use crate::ui::utils::adjust_brightness;
use gpui::{App, AppContext, Context, Entity, FontWeight, IntoElement, ParentElement, Render, Styled, Window, div, rgb};
use libera_reader_core::ctx::GlobalCTX;
use libera_reader_core::db::models::SettingsText;

pub(crate) struct SettingsPage {}
impl SettingsPage {
  pub(crate) fn new(cx: &mut App) -> Entity<Self> {
    cx.new(|_c| Self {})
  }
}

impl Render for SettingsPage {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = cx.ctx().settings.read().theme.data();
    div().w_full().h_full().flex().flex_col().text_color(rgb(adjust_brightness(theme.base_color_content, 1.2))).children([div()
      .bg(rgb(theme.base_100))
      .w_full()
      .h_full()
      .child(div().flex().flex_col().mt_2().pl_1_4().gap_y_3().children([div().children([
        div().font_weight(FontWeight::MEDIUM).text_xl().child(cx.i18n(SettingsText::UsedFormats)),
        div().mt_2().ml_5().children([
          div().flex().gap_2().items_center().children([
            Checkbox::new("pdf_checkbox").label("pdf").checked(cx.ctx().settings.read().pdf).on_click(cx.listener(|_this, _, _, cx| {
              cx.ctx_mut().settings.invert_pdf().unwrap();
              cx.notify();
            })),
            Checkbox::new("epub_checkbox").label("epub").checked(cx.ctx().settings.read().epub).on_click(cx.listener(|_this, _, _, cx| {
              cx.ctx_mut().settings.invert_epub().unwrap();
              cx.notify();
            })),
            Checkbox::new("mobi_checkbox").label("mobi").checked(cx.ctx().settings.read().mobi).on_click(cx.listener(|_this, _, _, cx| {
              cx.ctx_mut().settings.invert_mobi().unwrap();
              cx.notify();
            })),
          ]),
          div(),
        ]),
      ])]))])
  }
}
