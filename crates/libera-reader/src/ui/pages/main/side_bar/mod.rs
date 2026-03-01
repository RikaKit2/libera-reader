mod btn;

use crate::ui::pages::main::side_bar::btn::Btn;
use gpui::prelude::*;
use gpui::{App, Entity, IntoElement, ParentElement, Styled, Window, div};
use gpui_component::ActiveTheme;
use libera_reader_core::ctx::GlobalCTX;
use libera_reader_core::db::models::Route::{BookMarks, Favorite, FileManager, History, Library, Stats};
use libera_reader_core::db::models::{RootRoute, Route};

pub(crate) struct SideBar {}
impl SideBar {
  pub(crate) fn new(cx: &mut App) -> Entity<Self> {
    cx.new(|_| Self {})
  }
  fn mark_btn_as_active(&mut self, route: Route, cx: &mut App) {
    cx.ctx_mut().settings.set_route(RootRoute::Main(route)).unwrap();
  }
}
impl Render for SideBar {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    div().bg(cx.theme().border).w_12().h_full().flex().flex_col().justify_between().children([
      div().children([
        Btn::new(
          Library,
          "heroicons--book-open.svg",
          Some(Box::new({
            cx.listener(move |pages, _event, _window, cx| {
              pages.mark_btn_as_active(Library, cx);
              cx.notify();
            })
          })),
        ),
        Btn::new(
          FileManager,
          "heroicons--folder.svg",
          Some(Box::new({
            cx.listener(move |pages, _event, _window, cx| {
              pages.mark_btn_as_active(FileManager, cx);
              cx.notify();
            })
          })),
        ),
        Btn::new(
          History,
          "heroicons--clock.svg",
          Some(Box::new({
            cx.listener(move |pages, _event, _window, cx| {
              pages.mark_btn_as_active(History, cx);
              cx.notify();
            })
          })),
        ),
        Btn::new(
          Favorite,
          "heroicons--star.svg",
          Some(Box::new({
            cx.listener(move |pages, _event, _window, cx| {
              pages.mark_btn_as_active(Favorite, cx);
              cx.notify();
            })
          })),
        ),
        Btn::new(
          BookMarks,
          "heroicons--bookmark.svg",
          Some(Box::new({
            cx.listener(move |pages, _event, _window, cx| {
              pages.mark_btn_as_active(BookMarks, cx);
              cx.notify();
            })
          })),
        ),
      ]),
      div().children([
        Btn::new(
          Stats,
          "heroicons--chart-bar.svg",
          Some(Box::new({
            cx.listener(move |pages, _event, _window, cx| {
              pages.mark_btn_as_active(Stats, cx);
              cx.notify();
            })
          })),
        ),
        Btn::new(
          Route::Settings,
          "heroicons--cog-8-tooth.svg",
          Some(Box::new({
            cx.listener(move |pages, _event, _window, cx| {
              pages.mark_btn_as_active(Route::Settings, cx);
              cx.notify();
            })
          })),
        ),
      ]),
    ])
  }
}
