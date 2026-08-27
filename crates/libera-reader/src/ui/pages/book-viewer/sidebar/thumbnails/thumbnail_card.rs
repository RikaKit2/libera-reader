use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::ActiveTheme;

pub struct ThumbnailCard {
  page_number: usize,
  state: Entity<BookViewerState>,
}

impl ThumbnailCard {
  pub fn new(page_number: usize, state: Entity<BookViewerState>) -> Self {
    Self { page_number, state }
  }
}

impl Render for ThumbnailCard {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let current_page = self.state.read(cx).current_page;
    let is_active = current_page == self.page_number;
    let target_page = self.page_number;
    let state = self.state.clone();

    let border_color = if is_active { cx.theme().primary } else { cx.theme().border };

    div()
      .w_full()
      .flex()
      .flex_col()
      .items_center()
      .gap_y_1()
      .cursor_pointer()
      .on_mouse_down(
        MouseButton::Left,
        cx.listener(move |_this, _, _window, cx| {
          state.update(cx, |s, cx| {
            s.go_to_page(target_page);
            cx.notify();
          });
        }),
      )
      .child(
        div()
          .w(px(100.0))
          .h(px(140.0))
          .bg(cx.theme().secondary)
          .border_2()
          .border_color(border_color)
          .rounded_md()
          .shadow_sm()
          .flex()
          .items_center()
          .justify_center()
          .text_xs()
          .text_color(cx.theme().muted_foreground)
          .child(format!("Стр. {}", self.page_number)),
      )
      .child(
        div()
          .text_xs()
          .text_color(if is_active { cx.theme().primary } else { cx.theme().muted_foreground })
          .child(self.page_number.to_string()),
      )
  }
}
