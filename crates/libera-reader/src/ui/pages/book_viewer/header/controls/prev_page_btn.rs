use crate::ui::pages::book_viewer::constants::{RADIUS_SM, header};
use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{ActiveTheme, tooltip::Tooltip};
use rust_i18n::t;

#[derive(IntoElement)]
pub struct PrevPageBtn {
  state: Entity<BookViewerState>,
}

impl PrevPageBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl RenderOnce for PrevPageBtn {
  fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
    let theme = cx.theme();
    let can_go_prev = self.state.read(cx).current_page > 1;
    let state = self.state.clone();

    div()
      .id("header-prev-page-btn")
      .w(header::BTN_SIZE)
      .h(header::BTN_SIZE)
      .flex()
      .items_center()
      .justify_center()
      .rounded(RADIUS_SM)
      .when(can_go_prev, |s| {
        s.cursor_pointer().hover(move |s| s.bg(theme.foreground.opacity(0.08))).on_click(
          move |_, _window, cx| {
            state.update(cx, |s, cx| {
              s.prev_page();
              cx.notify();
            });
          },
        )
      })
      .tooltip(|window, cx| {
        Tooltip::new(t!("components.book_viewer.tooltips.prev_page").to_string()).build(window, cx)
      })
      .child(
        svg()
          .path("heroicons--chevron-up.svg")
          .size(header::ICON_SIZE_SM)
          .text_color(if can_go_prev { theme.foreground } else { theme.muted_foreground }),
      )
  }
}
