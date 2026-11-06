use crate::ui::pages::book_viewer::constants::{RADIUS_SM, header};
use crate::ui::pages::book_viewer::state::{BookViewerState, SidebarTab};
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{ActiveTheme, tooltip::Tooltip};
use rust_i18n::t;

#[derive(IntoElement)]
pub struct TtsBtn {
  state: Entity<BookViewerState>,
}

impl TtsBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl RenderOnce for TtsBtn {
  fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
    let theme = cx.theme();
    let active = self.state.read(cx).active_sidebar_tab == SidebarTab::Tts;
    let state = self.state.clone();

    div()
      .id("header-tts-btn")
      .w(header::BTN_SIZE)
      .h(header::BTN_SIZE)
      .flex()
      .items_center()
      .justify_center()
      .rounded(RADIUS_SM)
      .cursor_pointer()
      .tooltip(|window, cx| {
        Tooltip::new(t!("components.book_viewer.tooltips.tts").to_string()).build(window, cx)
      })
      .when(active, |s| s.bg(theme.primary.opacity(0.2)))
      .hover(move |s| s.bg(theme.foreground.opacity(0.08)))
      .on_click(move |_, _window, cx| {
        state.update(cx, |s, cx| {
          s.toggle_sidebar_tab(SidebarTab::Tts);
          cx.notify();
        });
      })
      .child(svg().path("ri--play-fill.svg").size(header::ICON_SIZE_SM).text_color(if active {
        theme.primary
      } else {
        theme.foreground
      }))
  }
}
