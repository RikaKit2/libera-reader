use crate::ui::pages::book_viewer::constants::sidebar;
use crate::ui::pages::book_viewer::state::{BookViewerState, SidebarTab};
use gpui::*;
use gpui_component::ActiveTheme;
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
    let theme = cx.theme();
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
      .h(sidebar::HEADER_HEIGHT)
      .px(sidebar::HEADER_PADDING_X)
      .bg(theme.title_bar)
      .border_b_1()
      .border_color(theme.border)
      .flex()
      .items_center()
      .justify_center()
      .child(div().text_sm().font_medium().text_color(theme.foreground).child(title))
  }
}
