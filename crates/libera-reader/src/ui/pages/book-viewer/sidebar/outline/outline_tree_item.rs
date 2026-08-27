use crate::ui::pages::book_viewer::state::{BookViewerState, OutlineItem};
use gpui::prelude::FluentBuilder;
use gpui::*;

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
          .py(px(2.0))
          .px(px(4.0))
          .rounded(px(3.0))
          .cursor_pointer()
          .hover(|s| s.bg(rgb(0x4A4A4D)))
          .when(is_active, |s| s.bg(rgb(0x4A4A4F)))
          .gap_x(px(4.0))
          .child(div().w(px(16.0)).flex().items_center().justify_center().when(has_children, |s| {
            s.child(
              div()
                .w(px(16.0))
                .h(px(16.0))
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
                .child(svg().path("ri--play-fill.svg").size(px(12.0)).text_color(rgb(0xD4D4D5))),
            )
          }))
          .child(
            div()
              .flex_1()
              .text_sm()
              .text_color(rgb(0xD4D4D5))
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
        s.child(div().pl(px(16.0)).children(
          children.into_iter().map(|child| OutlineTreeItem::new(child, state.clone(), cx)),
        ))
      })
  }
}
