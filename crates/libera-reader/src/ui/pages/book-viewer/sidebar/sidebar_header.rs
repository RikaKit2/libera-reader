use crate::ui::pages::book_viewer::state::{BookViewerState, SidebarTab};
use gpui::*;
use gpui_component::StyledExt;

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
      SidebarTab::Outline => "Содержание",
      SidebarTab::Bookmarks => "Закладки",
      SidebarTab::Thumbnails => "Обзор страниц",
      SidebarTab::Tts => "Преобразование текста в речь",
      SidebarTab::None => "",
    };

    let state = self.state.clone();

    div()
      .w_full()
      .h(px(32.0))
      .px(px(8.0))
      .bg(rgb(0x2A2A2E))
      .border_b_1()
      .border_color(rgb(0x4A4A4F))
      .flex()
      .items_center()
      .justify_between()
      .child(div().text_sm().font_medium().text_color(rgb(0xD4D4D5)).child(title))
      .child(
        div()
          .w(px(24.0))
          .h(px(24.0))
          .flex()
          .items_center()
          .justify_center()
          .rounded(px(3.0))
          .hover(|s| s.bg(rgb(0x666667)))
          .cursor_pointer()
          .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |_this, _, _window, cx| {
              state.update(cx, |s, cx| {
                s.active_sidebar_tab = SidebarTab::None;
                cx.notify();
              });
            }),
          )
          .child(
            svg().path("material-symbols--close.svg").size(px(16.0)).text_color(rgb(0xD4D4D5)),
          ),
      )
  }
}
