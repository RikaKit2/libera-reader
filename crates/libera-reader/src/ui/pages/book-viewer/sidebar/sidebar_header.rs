use crate::ui::pages::book_viewer::state::{BookViewerState, SidebarTab};
use gpui::*;
use gpui_component::StyledExt;
use rust_i18n::t;
pub struct SidebarHeader {
  state: Entity<BookViewerState>,
}

impl SidebarHeader {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl Render for SidebarHeader {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let tab = self.state.read(cx).active_sidebar_tab;
    let title = match tab {
      SidebarTab::Outline => t!("components.book_viewer.sidebar.outline_title").to_string(),
      SidebarTab::Bookmarks => t!("components.book_viewer.sidebar.bookmarks_title").to_string(),
      SidebarTab::Thumbnails => t!("components.book_viewer.sidebar.thumbnails_title").to_string(),
      SidebarTab::Tts => t!("components.book_viewer.sidebar.tts_title").to_string(),
      SidebarTab::None => "".to_string(),
    };

    div()
      .w_full()
      .h(px(32.0))
      .px(px(8.0))
      .bg(rgb(0x2A2A2E))
      .border_b_1()
      .border_color(rgb(0x4A4A4F))
      .flex()
      .items_center()
      .justify_center()
      .child(div().text_sm().font_medium().text_color(rgb(0xD4D4D5)).child(title))
  }
}
