use crate::ui::pages::book_viewer::constants::{RADIUS_SM, header};
use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::ActiveTheme;
use gpui_component::tooltip::Tooltip;
use rust_i18n::t;

pub struct ScrollModeBtn {
  state: Entity<BookViewerState>,
}

impl ScrollModeBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl Render for ScrollModeBtn {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = cx.theme();
    let is_continuous = self.state.read(cx).layout_mode.is_continuous();
    let state_entity = self.state.clone();

    div()
      .id("header-scroll-mode-btn")
      .w(header::BTN_SIZE)
      .h(header::BTN_SIZE)
      .flex()
      .items_center()
      .justify_center()
      .rounded(RADIUS_SM)
      .cursor_pointer()
      .tooltip(|window, cx| {
        Tooltip::new(t!("components.book_viewer.tooltips.scroll_mode").to_string())
          .build(window, cx)
      })
      .hover(move |s| s.bg(theme.foreground.opacity(0.08)))
      .when(is_continuous, |s| s.bg(theme.primary.opacity(0.15)))
      .on_mouse_down(
        MouseButton::Left,
        cx.listener(move |_this, _, _window, cx| {
          state_entity.update(cx, |s, cx| {
            s.layout_mode.toggle_continuous();
            cx.notify();
          });
        }),
      )
      .child(svg().path("pajamas--scroll-down.svg").size(header::ICON_SIZE_SM).text_color(
        match is_continuous {
          true => theme.primary,
          false => theme.foreground,
        },
      ))
  }
}
