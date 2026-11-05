use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::{
  Icon, Selectable, Sizable,
  button::{Button, ButtonVariants},
};
use rust_i18n::t;

#[derive(IntoElement)]
pub struct InvertColorsBtn {
  state: Entity<BookViewerState>,
}

impl InvertColorsBtn {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl RenderOnce for InvertColorsBtn {
  fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
    let inverted = self.state.read(cx).invert_colors;
    let state = self.state.clone();

    Button::new("header-invert-colors-btn")
      .ghost()
      .xsmall()
      .selected(inverted)
      .icon(Icon::new(Icon::empty()).path(if inverted {
        "heroicons--moon.svg"
      } else {
        "heroicons--sun-solid.svg"
      }))
      .tooltip(t!("components.book_viewer.tooltips.invert_colors").to_string())
      .on_click(move |_, _window, cx| {
        state.update(cx, |s, cx| {
          s.toggle_invert_colors();
          cx.notify();
        });
      })
  }
}
