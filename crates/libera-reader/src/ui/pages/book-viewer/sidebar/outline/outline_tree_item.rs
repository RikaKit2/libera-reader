use crate::ui::pages::book_viewer::constants::{RADIUS_SM, outline};
use crate::ui::pages::book_viewer::state::{BookViewerState, OutlineItem};
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::ActiveTheme;

pub struct OutlineTreeItem {
  item: OutlineItem,
  state: Entity<BookViewerState>,
  is_expanded: bool,
}

impl OutlineTreeItem {
  pub fn new(item: OutlineItem, state: Entity<BookViewerState>, cx: &mut App) -> Entity<Self> {
    cx.new(|_cx| Self { item, state, is_expanded: false })
  }
}

impl Render for OutlineTreeItem {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = cx.theme();
    let has_children = !self.item.children.is_empty();
    let current_page = self.state.read(cx).current_page;
    let is_active = self.item.page == current_page;
    let target_page = self.item.page;
    let state = self.state.clone();
    let is_expanded = self.is_expanded;

    div()
      .w_full()
      .flex()
      .flex_col()
      .child(
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
          .child(div().w(outline::TOGGLE_SIZE).flex().items_center().justify_center().when(
            has_children,
            |s| {
              s.child(
                div()
                  .w(outline::TOGGLE_SIZE)
                  .h(outline::TOGGLE_SIZE)
                  .flex()
                  .items_center()
                  .justify_center()
                  .cursor_pointer()
                  .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, _window, cx| {
                      this.is_expanded = !this.is_expanded;
                      cx.notify();
                    }),
                  )
                  .child(
                    svg()
                      .path("ri--play-fill.svg")
                      .size(outline::TOGGLE_ICON_SIZE)
                      .text_color(if is_active { theme.primary } else { theme.foreground }),
                  ),
              )
            },
          ))
          .child(
            div()
              .flex_1()
              .text_sm()
              .text_color(if is_active { theme.primary } else { theme.foreground })
              .truncate()
              .child(self.item.title.clone())
              .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |_this, _, _window, cx| {
                  state.update(cx, |s, cx| {
                    s.go_to_page(target_page);
                    cx.notify();
                  });
                }),
              ),
          ),
      )
      .when(has_children && is_expanded, |s| {
        let children = self.item.children.clone();
        let state = self.state.clone();
        s.child(div().pl(outline::CHILDREN_INDENT).children(
          children.into_iter().map(|child| OutlineTreeItem::new(child, state.clone(), cx)),
        ))
      })
  }
}
