use crate::ui::components::checkbox::Checkbox;
use crate::ui::utils::adjust_brightness;
use gpui::{div, rgb, App, AppContext, Context, Entity, FontWeight, IntoElement, ParentElement, Render, Styled, Window};
use libera_reader_core::db::models::TextId;
use libera_reader_core::ctx::GlobalCTX;

pub(crate) struct SettingsPage {
}
impl SettingsPage {
  pub(crate) fn new(cx: &mut App) -> Entity<Self> {
    cx.new(|_c| Self { })
  }
}

impl Render for SettingsPage {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = cx.ctx().theme.read().unwrap();
    let i18n = cx.ctx().settings.read().unwrap().language.clone();
    div().w_full().h_full().flex().flex_col().text_color(rgb(adjust_brightness(theme.base_color_content, 1.2))).children([
      div().bg(rgb(theme.base_100)).w_full().h_full().child(
        div().flex().flex_col().mt_2().pl_1_4().gap_y_3().children([
          div().children([
            div().font_weight(FontWeight::MEDIUM).text_xl().child(i18n.get(TextId::SettingsPageUsedFormats)),
            div().mt_2().ml_5().children([
              div().flex().gap_2().items_center().children([
                Checkbox::new("pdf_checkbox", cx.ctx().theme.clone())
                  .label("pdf")
                  .checked(cx.ctx().target_ext.read().unwrap().pdf)
                  .on_click(cx.listener(|_this, _, _, cx| {
                    cx.ctx().target_ext.write().unwrap().invert_pdf(&cx.ctx().db).unwrap();
                    cx.notify();
                  })),
                Checkbox::new("epub_checkbox", cx.ctx().theme.clone())
                  .label("epub")
                  .checked(cx.ctx().target_ext.read().unwrap().epub)
                  .on_click(cx.listener(|_this, _, _, cx| {
                    cx.ctx().target_ext.write().unwrap().invert_epub(&cx.ctx().db).unwrap();
                    cx.notify();
                  })),
                Checkbox::new("mobi_checkbox", cx.ctx().theme.clone())
                  .label("mobi")
                  .checked(cx.ctx().target_ext.read().unwrap().mobi)
                  .on_click(cx.listener(|_this, _, _, cx| {
                    cx.ctx().target_ext.write().unwrap().invert_mobi(&cx.ctx().db).unwrap();
                    cx.notify();
                  })),
              ]),
              div()
            ])
          ]),
        ])
      )
    ])
  }
}
