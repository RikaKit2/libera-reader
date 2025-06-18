mod btn;

use crate::ui::pages::main::side_bar::btn::Btn;
use crate::ui::pages::Pages;
use gpui::prelude::*;
use gpui::{div, rgb,  IntoElement, ParentElement, Styled, Window};
use libera_reader_core::db::models::Route::{BookMarks, Favorite, FileManager, History, Library, Stats};
use libera_reader_core::db::models::{RootRoute, Route};


impl Pages {
  fn mark_btn_as_active(&mut self, route: Route) {
    self.settings.write().unwrap().set_route(RootRoute::Main(route), &self.db);
  }

  pub fn render_main_page_side_bar(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = self.settings.read().unwrap().theme.clone();

    div()
      .bg(rgb(theme.base_300))
      .w_12()
      .h_full()
      .flex()
      .flex_col()
      .justify_between()
      .children([
        div().children([
          Btn::new(Library, self.settings.clone(), "heroicons--book-open.svg", Some(Box::new({
            cx.listener(move |pages, _event, _window, cx| {
              pages.mark_btn_as_active(Library);
              cx.notify();
            })
          }))),
          Btn::new(FileManager, self.settings.clone(), "heroicons--folder.svg", Some(Box::new({
            cx.listener(move |pages, _event, _window, cx| {
              pages.mark_btn_as_active(FileManager);
              cx.notify();
            })
          }))),
          Btn::new(History, self.settings.clone(), "heroicons--clock.svg", Some(Box::new({
            cx.listener(move |pages, _event, _window, cx| {
              pages.mark_btn_as_active(History);
              cx.notify();
            })
          }))),
          Btn::new(Favorite, self.settings.clone(), "heroicons--star.svg", Some(Box::new({
            cx.listener(move |pages, _event, _window, cx| {
              pages.mark_btn_as_active(Favorite);
              cx.notify();
            })
          }))),
          Btn::new(BookMarks, self.settings.clone(), "heroicons--bookmark.svg", Some(Box::new({
            cx.listener(move |pages, _event, _window, cx| {
              pages.mark_btn_as_active(BookMarks);
              cx.notify();
            })
          }))),
        ]),
        div().children([
          Btn::new(Stats, self.settings.clone(), "heroicons--chart-bar.svg", Some(Box::new({
            cx.listener(move |pages, _event, _window, cx| {
              pages.mark_btn_as_active(Stats);
              cx.notify();
            })
          }))),
          Btn::new(Route::Settings, self.settings.clone(), "heroicons--cog-8-tooth.svg", Some(Box::new({
            cx.listener(move |pages, _event, _window, cx| {
              pages.mark_btn_as_active(Route::Settings);
              cx.notify();
            })
          }))),
        ])
      ])
  }
}
