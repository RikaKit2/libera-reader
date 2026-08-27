use crate::ui::pages::book_viewer::constants::{RADIUS_SM, header};
use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::ActiveTheme;
use gpui_component::tooltip::Tooltip;
use rust_i18n::t;

pub struct FullscreenBtn {
  state: Entity<BookViewerState>,
}

impl FullscreenBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl Render for FullscreenBtn {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = cx.theme();
    let is_fullscreen = self.state.read(cx).is_fullscreen;
    let state = self.state.clone();

    div()
      .id("header-fullscreen-btn")
      .w(header::BTN_SIZE)
      .h(header::BTN_SIZE)
      .flex()
      .items_center()
      .justify_center()
      .rounded(RADIUS_SM)
      .cursor_pointer()
      .tooltip(|window, cx| {
        Tooltip::new(t!("components.book_viewer.tooltips.fullscreen").to_string()).build(window, cx)
      })
      .when(is_fullscreen, |s| s.bg(theme.primary.opacity(0.2)))
      .hover(move |s| s.bg(theme.foreground.opacity(0.08)))
      .on_mouse_down(
        MouseButton::Left,
        cx.listener(move |_this, _, window, cx| {
          state.update(cx, |s, cx| {
            s.is_fullscreen = !s.is_fullscreen;
            window.toggle_fullscreen();
            cx.notify();
          });
        }),
      )
      .child(
        svg()
          .path(if is_fullscreen {
            "gridicons--fullscreen-exit.svg"
          } else {
            "gridicons--fullscreen.svg"
          })
          .size(header::ICON_SIZE_MD)
          .text_color(if is_fullscreen { theme.primary } else { theme.foreground }),
      )
  }
}
