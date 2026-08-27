pub mod bookmarks;
pub mod outline;
pub mod sidebar_header;
pub mod thumbnails;
pub mod tts;

pub use bookmarks::BookmarksView;
pub use outline::OutlineView;
pub use sidebar_header::SidebarHeader;
pub use thumbnails::ThumbnailsView;
pub use tts::TtsView;

use crate::ui::pages::book_viewer::state::{BookViewerState, SidebarTab};
use gpui::*;

pub struct SideBar {
  state: Entity<BookViewerState>,
  header: Entity<SidebarHeader>,
  outline: Entity<OutlineView>,
  bookmarks: Entity<BookmarksView>,
  thumbnails: Entity<ThumbnailsView>,
  tts: Entity<TtsView>,
}

impl SideBar {
  pub fn new(window: &mut Window, cx: &mut App, state: Entity<BookViewerState>) -> Entity<Self> {
    let header = cx.new(|_cx| SidebarHeader::new(state.clone()));
    let outline = cx.new(|_cx| OutlineView::new(state.clone()));
    let bookmarks = BookmarksView::new(state.clone(), cx);
    let thumbnails = cx.new(|_cx| ThumbnailsView::new(state.clone()));
    let tts = TtsView::new(window, cx, state.clone());

    cx.new(|_cx| Self { state, header, outline, bookmarks, thumbnails, tts })
  }
}

impl Render for SideBar {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let active_tab = self.state.read(cx).active_sidebar_tab;

    if active_tab == SidebarTab::None {
      return div();
    }

    let content = match active_tab {
      SidebarTab::Outline => self.outline.clone().into_any_element(),
      SidebarTab::Bookmarks => self.bookmarks.clone().into_any_element(),
      SidebarTab::Thumbnails => self.thumbnails.clone().into_any_element(),
      SidebarTab::Tts => self.tts.clone().into_any_element(),
      SidebarTab::None => div().into_any_element(),
    };

    div()
      .w(px(250.0))
      .h_full()
      .bg(rgb(0x2A2A2E))
      .border_r_1()
      .border_color(rgb(0x4A4A4F))
      .flex()
      .flex_col()
      .child(self.header.clone())
      .child(div().w_full().flex_1().overflow_hidden().child(content))
  }
}
