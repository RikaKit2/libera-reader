use crate::ui::pages::book_viewer::constants::thumbnails;
use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::ActiveTheme;
use rust_i18n::t;
pub struct ThumbnailCard {
  pub page_number: usize,
  pub is_active: bool,
  pub image: Option<std::sync::Arc<RenderImage>>,
  pub state: Entity<BookViewerState>,
}

impl ThumbnailCard {
  pub fn new(
    page_number: usize, is_active: bool, image: Option<std::sync::Arc<RenderImage>>,
    state: Entity<BookViewerState>,
  ) -> Self {
    Self { page_number, is_active, image, state }
  }

  pub fn render(self, cx: &App) -> impl IntoElement {
    let theme = cx.theme();
    let is_active = self.is_active;
    let target_page = self.page_number;
    let state = self.state.clone();

    let border_color = if is_active { theme.primary } else { theme.border };

    let image_element = if let Some(img) = self.image {
      div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .overflow_hidden()
        .child(gpui::img(img).size_full().object_fit(gpui::ObjectFit::Contain))
        .into_any_element()
    } else {
      div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .text_xs()
        .text_color(theme.muted_foreground)
        .child(
          t!("components.book_viewer.thumbnails.page_label", page = self.page_number).to_string(),
        )
        .into_any_element()
    };

    div()
      .w(thumbnails::CARD_WIDTH)
      .flex()
      .flex_col()
      .items_center()
      .gap_y_1()
      .cursor_pointer()
      .on_mouse_down(MouseButton::Left, move |_event, _window, cx| {
        state.update(cx, |s, cx| {
          s.go_to_page(target_page);
          cx.notify();
        });
      })
      .child(
        div()
          .w(thumbnails::CARD_WIDTH)
          .h(thumbnails::CARD_HEIGHT)
          .bg(theme.secondary)
          .border_2()
          .border_color(border_color)
          .rounded_md()
          .shadow_sm()
          .overflow_hidden()
          .child(image_element),
      )
      .child(
        div()
          .text_xs()
          .text_color(if is_active { theme.primary } else { theme.muted_foreground })
          .child(self.page_number.to_string()),
      )
  }
}
