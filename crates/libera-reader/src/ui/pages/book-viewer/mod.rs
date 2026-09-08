#![allow(dead_code)]

pub mod cache;
pub mod constants;
pub mod document;
pub mod header;
pub mod image_utils;
pub mod loader;
pub mod sidebar;
pub mod state;
pub mod viewport;

pub use cache::{BookViewerCache, PageImageState, PageLinksState, PageTextState};
pub use document::DocumentData;
pub use header::Header;
pub use sidebar::SideBar;
pub use state::BookViewerState;
pub use viewport::Viewport;

use crate::app_ext::AppExt;
use gpui::prelude::*;
use gpui::{App, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div};

pub(crate) struct BookViewer {
  state: Entity<BookViewerState>,
  header: Entity<Header>,
  sidebar: Entity<SideBar>,
  viewport: Entity<Viewport>,
}

impl BookViewer {
  pub(crate) fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
    let state = cx.book_viewer_state().clone();
    let header = Header::new(window, cx, state.clone());
    let sidebar = SideBar::new(window, cx, state.clone());
    let viewport = Viewport::new(state.clone(), cx);

    cx.new(|_cx| Self { state, header, sidebar, viewport })
  }

  pub(crate) fn state(&self) -> &Entity<BookViewerState> {
    &self.state
  }
}

impl Render for BookViewer {
  fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    div().size_full().flex().flex_col().child(self.header.clone()).child(
      div()
        .flex_1()
        .flex()
        .w_full()
        .overflow_hidden()
        .child(self.sidebar.clone())
        .child(self.viewport.clone()),
    )
  }
}
