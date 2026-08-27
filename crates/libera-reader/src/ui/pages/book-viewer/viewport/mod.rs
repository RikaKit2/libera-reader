pub mod page;
pub mod paged_container;
pub mod scroll_container;

pub use paged_container::PagedContainer;
pub use scroll_container::ScrollContainer;

use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::ActiveTheme;

pub struct Viewport {
  state: Entity<BookViewerState>,
  scroll_container: Entity<ScrollContainer>,
  paged_container: Entity<PagedContainer>,
}

impl Viewport {
  pub fn new(state: Entity<BookViewerState>, cx: &mut App) -> Entity<Self> {
    let scroll_container = cx.new(|_cx| ScrollContainer::new(state.clone()));
    let paged_container = cx.new(|_cx| PagedContainer::new(state.clone()));

    cx.new(|_cx| Self { state, scroll_container, paged_container })
  }
}

impl Render for Viewport {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let is_continuous = self.state.read(cx).layout_mode.is_continuous();
    let bg = cx.theme().background;

    div().size_full().bg(bg).flex_1().overflow_hidden().child(if is_continuous {
      self.scroll_container.clone().into_any_element()
    } else {
      self.paged_container.clone().into_any_element()
    })
  }
}
