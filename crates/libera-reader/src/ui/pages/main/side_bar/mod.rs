mod btn;

use crate::ui::pages::main::side_bar::btn::Btn;
use crate::ui::pages::CTX;
use gpui::prelude::*;
use gpui::{div, rgb, App, Entity, IntoElement, ParentElement, Styled, Window};
use libera_reader_core::db::models::Route::{BookMarks, Favorite, FileManager, History, Library, Stats};
use libera_reader_core::db::models::{RootRoute, Route};
use std::sync::Arc;


pub(crate) struct SideBar {
  ctx: Arc<CTX>,
}
impl SideBar {
  pub(crate) fn new(cx: &mut App, ctx: Arc<CTX>) -> Entity<Self> {
    cx.new(|_| Self { ctx })
  }
  fn mark_btn_as_active(&mut self, route: Route) {
    self.ctx.settings.write().unwrap().set_route(RootRoute::Main(route), &self.ctx.db);
  }
}
impl Render for SideBar {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = self.ctx.theme.read().unwrap();
    div()
      .bg(rgb(theme.base_300))
      .w_12()
      .h_full()
      .flex()
      .flex_col()
      .justify_between()
      .children([
        div().children([
          Btn::new(Library, self.ctx.settings.clone(), self.ctx.theme.clone(), "heroicons--book-open.svg", Some(Box::new({
            cx.listener(move |pages, _event, _window, cx| {
              pages.mark_btn_as_active(Library);
              cx.notify();
            })
          }))),
          Btn::new(FileManager, self.ctx.settings.clone(), self.ctx.theme.clone(), "heroicons--folder.svg", Some(Box::new({
            cx.listener(move |pages, _event, _window, cx| {
              pages.mark_btn_as_active(FileManager);
              cx.notify();
            })
          }))),
          Btn::new(History, self.ctx.settings.clone(), self.ctx.theme.clone(), "heroicons--clock.svg", Some(Box::new({
            cx.listener(move |pages, _event, _window, cx| {
              pages.mark_btn_as_active(History);
              cx.notify();
            })
          }))),
          Btn::new(Favorite, self.ctx.settings.clone(), self.ctx.theme.clone(), "heroicons--star.svg", Some(Box::new({
            cx.listener(move |pages, _event, _window, cx| {
              pages.mark_btn_as_active(Favorite);
              cx.notify();
            })
          }))),
          Btn::new(BookMarks, self.ctx.settings.clone(), self.ctx.theme.clone(), "heroicons--bookmark.svg", Some(Box::new({
            cx.listener(move |pages, _event, _window, cx| {
              pages.mark_btn_as_active(BookMarks);
              cx.notify();
            })
          }))),
        ]),
        div().children([
          Btn::new(Stats, self.ctx.settings.clone(), self.ctx.theme.clone(), "heroicons--chart-bar.svg", Some(Box::new({
            cx.listener(move |pages, _event, _window, cx| {
              pages.mark_btn_as_active(Stats);
              cx.notify();
            })
          }))),
          Btn::new(Route::Settings, self.ctx.settings.clone(), self.ctx.theme.clone(), "heroicons--cog-8-tooth.svg", Some(Box::new({
            cx.listener(move |pages, _event, _window, cx| {
              pages.mark_btn_as_active(Route::Settings);
              cx.notify();
            })
          }))),
        ])
      ])
  }
}
