use crate::ui::utils::adjust_brightness;
use gpui::prelude::FluentBuilder;
use gpui::{div, rgb, App, AppContext, Context, Entity, FontWeight, InteractiveElement, IntoElement,
           MouseButton, MouseDownEvent, ParentElement, Render, Styled, Window};
use libera_reader_core::ctx::GlobalCTX;
use libera_reader_core::db::models::{RootRoute, Route, TextId};
use rfd::FileDialog;

pub(crate) struct SetupPage {
  window_of_selecting_folder_is_open: bool,
}
impl SetupPage {
  pub(crate) fn new(cx: &mut App) -> Entity<Self> {
    cx.new(|_| Self { window_of_selecting_folder_is_open: false })
  }
  fn select_folder(&mut self, _event: &MouseDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
    match self.window_of_selecting_folder_is_open {
      true => {}
      false => {
        match FileDialog::new().pick_folder() {
          None => {}
          Some(path) => {
            cx.ctx().settings.write().unwrap().set_path_to_scan(path.display().to_string(), &cx.ctx().db).unwrap();
            self.window_of_selecting_folder_is_open = true;
            cx.notify();
          }
        }
      }
    }
  }
  fn handler_for_next_btn(&mut self, _event: &MouseDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
    let path_to_scan = cx.ctx().settings.read().unwrap().path_to_scan.is_some();
    match path_to_scan {
      true => {
        cx.ctx().settings.write().unwrap().set_setup_status(true, &cx.ctx().db).unwrap();
        cx.ctx().settings.write().unwrap().set_route(RootRoute::Main(Route::Library), &cx.ctx().db).unwrap();
        cx.notify();
      }
      false => {}
    }
  }
}
impl Render for SetupPage {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = &cx.ctx().theme.read().unwrap();
    let i18n = cx.ctx().settings.read().unwrap().language.clone();
    div().bg(rgb(theme.base_100)).w_full().h_full().p_6().flex().flex_col().justify_between().text_color(rgb(theme.base_color_content)).children(
      [
        div().flex().flex_col().children([
          div().child(i18n.get(TextId::SetupPageTitle)).font_weight(FontWeight::BOLD),
          div().flex_col().children([
            div().gap_2().flex().children([
              div().child(i18n.get(TextId::MessageOfSelectingTargetDir)),
              div().flex().px_1()
                .rounded_sm()
                .on_mouse_down(MouseButton::Left, cx.listener(Self::select_folder))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(theme.primary_content_color))
                .bg(rgb(adjust_brightness(theme.primary_color, 0.9)))
                .hover(|s| s.bg(rgb(adjust_brightness(theme.primary_color, 1.1))))
                .child(i18n.get(TextId::SetupPageSelectBtn)),
            ]),
            div().flex_col().children([
              div().child(i18n.get(TextId::TargetPath)),
              div().when(cx.ctx().settings.read().unwrap().path_to_scan.is_some(),
                         |_| div().child(cx.ctx().settings.read().unwrap().path_to_scan.clone().unwrap()))
            ]),
          ]),
        ]),
        div().w_full().flex().justify_end().children([
          div()
            .px_2()
            .text_color(rgb(theme.info_content_color)).rounded_sm()
            .font_weight(FontWeight::BOLD)
            .bg(rgb(adjust_brightness(theme.info_color, 0.9)))
            .hover(|s| s.bg(rgb(adjust_brightness(theme.info_color, 1.1))))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::handler_for_next_btn))
            .child(i18n.get(TextId::SetupPageNextBtn))
        ]),
      ]
    )
  }
}

