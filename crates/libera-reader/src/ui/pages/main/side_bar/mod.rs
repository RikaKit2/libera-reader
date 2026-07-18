mod btn;

use crate::ui::pages::main::side_bar::btn::Btn;
use gpui::prelude::*;
use gpui::{App, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div};
use gpui_component::ActiveTheme;
use libera_reader_core::ctx::GlobalCTX;
use libera_reader_core::db::models::{RootRoute, Route};

/// Primary navigation entries shown at the top of the sidebar.
const TOP_ENTRIES: &[(Route, &str)] = &[
  (Route::Library, "heroicons--book-open.svg"),
  (Route::FileManager, "heroicons--folder.svg"),
  (Route::History, "heroicons--clock.svg"),
  (Route::Favorite, "heroicons--star.svg"),
  (Route::BookMarks, "heroicons--bookmark.svg"),
];

/// Secondary entries pinned to the bottom of the sidebar.
const BOTTOM_ENTRIES: &[(Route, &str)] =
  &[(Route::Stats, "heroicons--chart-bar.svg"), (Route::Settings, "heroicons--cog-8-tooth.svg")];

pub(crate) struct SideBar {}

impl SideBar {
  pub(crate) fn new(cx: &mut App) -> Entity<Self> {
    cx.new(|_| Self {})
  }

  fn mark_btn_as_active(&mut self, route: Route, cx: &mut App) {
    cx.ctx_mut().settings.set_route(RootRoute::Main(route)).unwrap();
  }

  /// Build a sidebar button that activates the given route on click.
  /// One shared click handler for every entry — no per-button duplication.
  fn entry(&self, route: Route, icon: &'static str, cx: &mut Context<Self>) -> Btn {
    Btn::new(
      route,
      icon,
      Some(Box::new(cx.listener(move |this, _event, _window, cx| {
        this.mark_btn_as_active(route, cx);
        cx.notify();
      }))),
    )
  }
}

impl Render for SideBar {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    div().bg(cx.theme().border).w_12().h_full().flex().flex_col().justify_between().children([
      div().children(TOP_ENTRIES.iter().map(|&(route, icon)| self.entry(route, icon, cx))),
      div().children(BOTTOM_ENTRIES.iter().map(|&(route, icon)| self.entry(route, icon, cx))),
    ])
  }
}
