use gpui::*;
use gpui_component::{ActiveTheme, Icon, button::*};

use crate::app_ext::AppExt;
use crate::books_state::{BooksState, TargetList};
/// Standalone "reverse sort direction" button for the top bar.
///
/// Renders an arrow icon (up/down depending on the current sort direction) that
/// toggles the reverse flag on the shared books state for a given target list.
/// Extracted from `SortControls` so its state/theming lives in its own
/// component, while still being placed between the sort and mode selects.
pub struct ReverseBtn {
  books_state: Entity<BooksState>,
  target: TargetList,
}

impl ReverseBtn {
  pub fn new(target: TargetList, cx: &mut App) -> Entity<Self> {
    let books_state = cx.books_state_entity().clone();
    cx.new(|cx| {
      // Re-render whenever the sort direction flips so the arrow icon updates.
      cx.observe(&books_state, |_, _, cx| cx.notify()).detach();
      Self { books_state, target }
    })
  }

  fn is_reversed(&self, cx: &App) -> bool {
    self.books_state.read(cx).is_reversed(self.target)
  }
}

impl Render for ReverseBtn {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let dir_icon = if self.is_reversed(cx) { "a-arrow-down.svg" } else { "a-arrow-up.svg" };

    Button::new("reverse_btn")
      .icon(Icon::new(Icon::empty()).path(dir_icon))
      .text_color(cx.theme().foreground)
      .bg(cx.theme().background)
      .on_click({
        let books_state = self.books_state.clone();
        let target = self.target;
        move |_ev, _window, cx| {
          books_state.update(cx, |state, cx| state.toggle_reverse(target, cx));
        }
      })
  }
}
