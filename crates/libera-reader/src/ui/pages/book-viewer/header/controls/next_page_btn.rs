use crate::ui::pages::book_viewer::constants::{RADIUS_SM, header};
use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::ActiveTheme;
use gpui_component::tooltip::Tooltip;
use rust_i18n::t;

pub struct NextPageBtn {
  state: Entity<BookViewerState>,
}

impl NextPageBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl Render for NextPageBtn {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = cx.theme();
    let state_ref = self.state.read(cx);
    let can_go_next = state_ref.current_page < state_ref.total_pages;
    let state = self.state.clone();

    div()
      .id("header-next-page-btn")
      .w(header::BTN_SIZE)
      .h(header::BTN_SIZE)
      .flex()
      .items_center()
      .justify_center()
      .rounded(RADIUS_SM)
      .cursor_pointer()
      .tooltip(|window, cx| {
        Tooltip::new(t!("components.book_viewer.tooltips.next_page").to_string()).build(window, cx)
      })
      .hover(move |s| s.bg(theme.foreground.opacity(0.08)))
      .on_mouse_down(
        MouseButton::Left,
        cx.listener(move |_this, _, _window, cx| {
          state.update(cx, |s, cx| {
            s.next_page();
            cx.notify();
          });
        }),
      )
      .child(
        svg()
          .path("heroicons--chevron-down.svg")
          .size(header::ICON_SIZE_SM)
          .text_color(if can_go_next { theme.foreground } else { theme.muted_foreground }),
      )
  }
}
