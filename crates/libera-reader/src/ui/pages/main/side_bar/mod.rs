mod btn;

use crate::ui::pages::main::side_bar::btn::Btn;
use gpui::prelude::*;
use gpui::{div, rgb, App, Entity, IntoElement, ParentElement, Styled, Window};
use libera_reader_core::ctx::GlobalCTX;
use libera_reader_core::db::models::Route::{BookMarks, Favorite, FileManager, History, Library, Stats};
use libera_reader_core::db::models::{RootRoute, Route};

pub(crate) struct SideBar {}
impl SideBar {
  pub(crate) fn new(cx: &mut App) -> Entity<Self> {
    cx.new(|_| Self {})
  }
  fn mark_btn_as_active(&mut self, route: Route, cx: &mut App) {
    cx.ctx().settings.write().unwrap().set_route(RootRoute::Main(route), &cx.ctx().db).unwrap();
  }
}
impl Render for SideBar {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = cx.ctx().theme.read().unwrap();
    div().bg(rgb(theme.base_300)).w_12().h_full().flex().flex_col().justify_between().children([
      div().children([
        Btn::new(Library, cx.ctx().settings.clone(), cx.ctx().theme.clone(), "heroicons--book-open.svg", Some(Box::new({
          cx.listener(move |pages, _event, _window, cx| {
            pages.mark_btn_as_active(Library, cx);
            cx.notify();
          })
        }))),
        Btn::new(FileManager, cx.ctx().settings.clone(), cx.ctx().theme.clone(), "heroicons--folder.svg", Some(Box::new({
          cx.listener(move |pages, _event, _window, cx| {
            pages.mark_btn_as_active(FileManager, cx);
            cx.notify();
          })
        }))),
        Btn::new(History, cx.ctx().settings.clone(), cx.ctx().theme.clone(), "heroicons--clock.svg", Some(Box::new({
          cx.listener(move |pages, _event, _window, cx| {
            pages.mark_btn_as_active(History, cx);
            cx.notify();
          })
        }))),
        Btn::new(Favorite, cx.ctx().settings.clone(), cx.ctx().theme.clone(), "heroicons--star.svg", Some(Box::new({
          cx.listener(move |pages, _event, _window, cx| {
            pages.mark_btn_as_active(Favorite, cx);
            cx.notify();
          })
        }))),
        Btn::new(BookMarks, cx.ctx().settings.clone(), cx.ctx().theme.clone(), "heroicons--bookmark.svg", Some(Box::new({
          cx.listener(move |pages, _event, _window, cx| {
            pages.mark_btn_as_active(BookMarks, cx);
            cx.notify();
          })
        }))),
      ]),
      div().children([
        Btn::new(Stats, cx.ctx().settings.clone(), cx.ctx().theme.clone(), "heroicons--chart-bar.svg", Some(Box::new({
          cx.listener(move |pages, _event, _window, cx| {
            pages.mark_btn_as_active(Stats, cx);
            cx.notify();
          })
        }))),
        Btn::new(Route::Settings, cx.ctx().settings.clone(), cx.ctx().theme.clone(), "heroicons--cog-8-tooth.svg", Some(Box::new({
          cx.listener(move |pages, _event, _window, cx| {
            pages.mark_btn_as_active(Route::Settings, cx);
            cx.notify();
          })
        }))),
      ])
    ])
  }
}
