use crate::ui::pages::book_viewer::constants::{RADIUS_SM, header};
use crate::ui::pages::book_viewer::state::{BookViewerState, SidebarTab};
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::ActiveTheme;
use gpui_component::tooltip::Tooltip;
use rust_i18n::t;

pub struct OutlineBtn {
  state: Entity<BookViewerState>,
}

impl OutlineBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl Render for OutlineBtn {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = cx.theme();
    let active = self.state.read(cx).active_sidebar_tab == SidebarTab::Outline;
    let state = self.state.clone();

    div()
      .id("header-outline-btn")
      .w(header::BTN_SIZE)
      .h(header::BTN_SIZE)
      .flex()
      .items_center()
      .justify_center()
      .rounded(RADIUS_SM)
      .cursor_pointer()
      .tooltip(|window, cx| {
        Tooltip::new(t!("components.book_viewer.tooltips.outline").to_string()).build(window, cx)
      })
      .when(active, |s| s.bg(theme.primary.opacity(0.2)))
      .hover(move |s| s.bg(theme.foreground.opacity(0.08)))
      .on_mouse_down(
        MouseButton::Left,
        cx.listener(move |_this, _, _window, cx| {
          state.update(cx, |s, cx| {
            s.toggle_sidebar_tab(SidebarTab::Outline);
            cx.notify();
          });
        }),
      )
      .child(
        svg().path("material-symbols--list.svg").size(header::ICON_SIZE_LG).text_color(if active {
          theme.primary
        } else {
          theme.foreground
        }),
      )
  }
}
