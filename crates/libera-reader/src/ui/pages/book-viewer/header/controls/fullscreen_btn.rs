use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::{
  Icon, Selectable, Sizable,
  button::{Button, ButtonVariants},
};
use rust_i18n::t;

#[derive(IntoElement)]
pub struct FullscreenBtn {
  state: Entity<BookViewerState>,
}

impl FullscreenBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl RenderOnce for FullscreenBtn {
  fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
    let is_fullscreen = self.state.read(cx).is_fullscreen;
    let state = self.state.clone();

    Button::new("header-fullscreen-btn")
      .ghost()
      .xsmall()
      .selected(is_fullscreen)
      .icon(Icon::new(Icon::empty()).path(if is_fullscreen {
        "gridicons--fullscreen-exit.svg"
      } else {
        "gridicons--fullscreen.svg"
      }))
      .tooltip(t!("components.book_viewer.tooltips.fullscreen").to_string())
      .on_click(move |_, window, cx| {
        state.update(cx, |s, cx| {
          s.is_fullscreen = !s.is_fullscreen;
          window.toggle_fullscreen();
          cx.notify();
        });
      })
  }
}
