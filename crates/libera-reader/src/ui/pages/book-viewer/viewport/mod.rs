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
    let scroll_container = ScrollContainer::new(state.clone(), cx);
    let paged_container = PagedContainer::new(state.clone(), cx);

    cx.new(|_cx| Self { state, scroll_container, paged_container })
  }

  pub fn scroll_to_page(&self, page: usize, cx: &mut App) {
    self.scroll_container.update(cx, |sc, _cx| {
      sc.scroll_to_page(page);
    });
  }
}

impl Render for Viewport {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let is_continuous = self.state.read(cx).layout_mode.is_continuous();
    let bg = cx.theme().background;

    div().size_full().bg(bg).flex_1().overflow_hidden().child(match is_continuous {
      true => self.scroll_container.clone().into_any_element(),
      false => self.paged_container.clone().into_any_element(),
    })
  }
}
