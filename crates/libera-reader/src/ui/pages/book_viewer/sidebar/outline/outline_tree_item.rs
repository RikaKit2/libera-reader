use crate::ui::pages::book_viewer::constants::{RADIUS_SM, outline};
use crate::ui::pages::book_viewer::state::{BookViewerState, OutlineItem};
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::ActiveTheme;

pub struct OutlineTreeItem {
  item: OutlineItem,
  item_id: usize,
  state: Entity<BookViewerState>,
}

impl OutlineTreeItem {
  pub fn new(item: OutlineItem, item_id: usize, state: Entity<BookViewerState>) -> Self {
    Self { item, item_id, state }
  }

  pub fn render_tree(
    item: OutlineItem, item_id: usize, state: Entity<BookViewerState>, cx: &App,
  ) -> AnyElement {
    let has_children = !item.children.is_empty();
    let state_read = state.read(cx);
    let current_page = state_read.current_page;
    let is_active = item.page == current_page;
    let target_page = item.page;
    let is_expanded = state_read.expanded_outline_items.contains(&item_id);
    let state_entity = state.clone();

    let toggle_button = if has_children {
      let state_toggle = state.clone();
      div()
        .w(outline::TOGGLE_SIZE)
        .h(outline::TOGGLE_SIZE)
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
          window.prevent_default();
          state_toggle.update(cx, |s, cx| {
            if s.expanded_outline_items.contains(&item_id) {
              s.expanded_outline_items.shift_remove(&item_id);
            } else {
              s.expanded_outline_items.insert(item_id);
            }
            cx.notify();
          });
        })
        .child(
          svg()
            .path(if is_expanded { "heroicons--chevron-down.svg" } else { "ri--play-fill.svg" })
            .size(outline::TOGGLE_ICON_SIZE)
            .text_color(if is_active { cx.theme().primary } else { cx.theme().foreground }),
        )
        .into_any_element()
    } else {
      div().w(outline::TOGGLE_SIZE).into_any_element()
    };

    let theme = cx.theme();

    let mut row = div().w_full().flex().flex_col().child(
      div()
        .w_full()
        .flex()
        .items_center()
        .py(outline::ITEM_PY)
        .px(outline::ITEM_PX)
        .rounded(RADIUS_SM)
        .cursor_pointer()
        .hover(move |s| s.bg(theme.foreground.opacity(0.08)))
        .when(is_active, |s| s.bg(theme.primary.opacity(0.15)))
        .gap_x(outline::ITEM_GAP_X)
        .child(
          div().w(outline::TOGGLE_SIZE).flex().items_center().justify_center().child(toggle_button),
        )
        .child(
          div()
            .flex_1()
            .text_sm()
            .text_color(if is_active { theme.primary } else { theme.foreground })
            .truncate()
            .child(item.title.clone())
            .on_mouse_down(MouseButton::Left, move |_, _window, cx| {
              state_entity.update(cx, |s, cx| {
                s.go_to_page(target_page);
                cx.notify();
              });
            }),
        ),
    );

    if has_children && is_expanded {
      let mut child_elements = Vec::new();
      for (child_idx, child) in item.children.into_iter().enumerate() {
        let child_id = item_id.wrapping_mul(31).wrapping_add(child_idx + 1);
        child_elements.push(Self::render_tree(child, child_id, state.clone(), cx));
      }
      row = row.child(div().pl(outline::CHILDREN_INDENT).children(child_elements));
    }

    row.into_any_element()
  }
}

impl Render for OutlineTreeItem {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    Self::render_tree(self.item.clone(), self.item_id, self.state.clone(), cx)
  }
}
