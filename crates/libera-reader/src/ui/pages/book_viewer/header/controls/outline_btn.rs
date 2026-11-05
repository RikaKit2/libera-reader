use crate::ui::pages::book_viewer::state::{BookViewerState, SidebarTab};
use gpui::*;
use gpui_component::{
  Icon, Selectable, Sizable,
  button::{Button, ButtonVariants},
};
use rust_i18n::t;

#[derive(IntoElement)]
pub struct OutlineBtn {
  state: Entity<BookViewerState>,
}

impl OutlineBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl RenderOnce for OutlineBtn {
  fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
    let active = self.state.read(cx).active_sidebar_tab == SidebarTab::Outline;
    let state = self.state.clone();

    Button::new("header-outline-btn")
      .ghost()
      .xsmall()
      .selected(active)
      .icon(Icon::new(Icon::empty()).path("material-symbols--list.svg"))
      .tooltip(t!("components.book_viewer.tooltips.outline").to_string())
      .on_click(move |_, _window, cx| {
        state.update(cx, |s, cx| {
          s.toggle_sidebar_tab(SidebarTab::Outline);
          cx.notify();
        });
      })
  }
}
