use crate::ui::utils::adjust_brightness;
use gpui::prelude::FluentBuilder;
use gpui::{
  App, AppContext, Context, Entity, FontWeight, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
  ParentElement, Render, Styled, Window, div,
};
use gpui_component::ActiveTheme;
use libera_reader_core::ctx::GlobalCTX;
use libera_reader_core::db::models::{RootRoute, Route, SetupText};
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
      false => match FileDialog::new().pick_folder() {
        None => {}
        Some(path) => {
          cx.ctx_mut().settings.set_path_to_scan(path).unwrap();
          self.window_of_selecting_folder_is_open = true;
          cx.notify();
        }
      },
    }
  }
  fn handler_for_next_btn(&mut self, _event: &MouseDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
    let path_to_scan = cx.ctx().settings.read().path_to_scan.is_some();
    if path_to_scan {
      cx.ctx_mut().settings.set_setup_status(true).unwrap();
      cx.ctx_mut().settings.set_route(RootRoute::Main(Route::Library)).unwrap();
      cx.notify();
    }
  }
}
impl Render for SetupPage {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = cx.theme();
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
        div().flex().flex_col().children([
          div().child(cx.i18n(SetupText::Title)).font_weight(FontWeight::BOLD),
          div().flex_col().children([
            div().gap_2().flex().children([
              div().child(cx.i18n(SetupText::TargetDir)),
              div()
                .flex()
                .px_1()
                .rounded_sm()
                .on_mouse_down(MouseButton::Left, cx.listener(Self::select_folder))
                .font_weight(FontWeight::BOLD)
                .text_color(theme.primary_foreground)
                .bg(adjust_brightness(theme.primary, 0.9))
                .hover(|s| s.bg(adjust_brightness(theme.primary, 1.1)))
                .child(cx.i18n(SetupText::SelectBtn)),
            ]),
            div().flex_col().children([
              div().child(cx.i18n(SetupText::TargetPath)),
              div().when(cx.ctx().settings.read().path_to_scan.is_some(), |_| {
                div().child(cx.ctx().settings.get_path_to_scan_str().unwrap())
              }),
            ]),
          ]),
        ]),
        div().w_full().flex().justify_end().children([div()
          .px_2()
          .text_color(theme.info)
          .rounded_sm()
          .font_weight(FontWeight::BOLD)
          .bg(adjust_brightness(theme.info, 0.9))
          .hover(|s| s.bg(adjust_brightness(theme.info, 1.1)))
          .on_mouse_down(MouseButton::Left, cx.listener(Self::handler_for_next_btn))
          .child(cx.i18n(SetupText::NextBtn))]),
      ])
  }
}
