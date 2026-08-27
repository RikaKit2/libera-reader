pub mod outline_tree_item;

pub use outline_tree_item::OutlineTreeItem;

use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::ActiveTheme;
use gpui_component::scroll::ScrollableElement;

pub struct OutlineView {
  state: Entity<BookViewerState>,
}

impl OutlineView {
  pub fn new(state: Entity<BookViewerState>) -> Self {
    Self { state }
  }
}

impl Render for OutlineView {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let outline = self.state.read(cx).outline.clone();
    let state = self.state.clone();

    if outline.is_empty() {
      return div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .p_4()
        .text_sm()
        .text_color(cx.theme().muted_foreground)
        .child("Оглавление отсутствует в этом документе")
        .into_any_element();
    }

    div()
      .size_full()
      .overflow_y_scrollbar()
      .p_2()
      .flex()
      .flex_col()
      .gap_y_1()
      .children(outline.into_iter().map(|item| OutlineTreeItem::new(item, state.clone(), cx)))
      .into_any_element()
  }
}
