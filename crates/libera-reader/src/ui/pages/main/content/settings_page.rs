use crate::ui::utils::adjust_brightness;
use gpui::{App, AppContext, Context, Entity, FontWeight, IntoElement, ParentElement, Render, Styled, Window, div};
use gpui_component::ActiveTheme;
use gpui_component::checkbox::Checkbox;
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
    let text_color = adjust_brightness(cx.theme().foreground, 1.2);
    let bg_color = cx.theme().background;
    let guard = cx.ctx().settings.read();
    let pdf_status = guard.pdf;
    let epub_status = guard.epub;
    let mobi_status = guard.mobi;
    div().w_full().h_full().flex().flex_col().text_color(text_color).children([div()
      .bg(bg_color)
      .w_full()
      .h_full()
      .child(div().flex().flex_col().mt_2().pl_1_4().gap_y_3().children([div().children([
        div().font_weight(FontWeight::MEDIUM).text_xl().child(cx.i18n(SettingsText::UsedFormats)),
        div().mt_2().ml_5().children([
          div().flex().gap_2().items_center().children([
            Checkbox::new("pdf_checkbox").label("pdf").checked(pdf_status).on_click(|_, _, cx| {
              cx.ctx_mut().settings.invert_pdf().unwrap();
            }),
            Checkbox::new("epub_checkbox").label("epub").checked(epub_status).on_click(|_, _, cx| {
              cx.ctx_mut().settings.invert_pdf().unwrap();
            }),
            Checkbox::new("mobi_checkbox").label("mobi").checked(mobi_status).on_click(|_, _, cx| {
              cx.ctx_mut().settings.invert_pdf().unwrap();
            }),
          ]),
          div(),
        ]),
      ])]))])
  }
}
