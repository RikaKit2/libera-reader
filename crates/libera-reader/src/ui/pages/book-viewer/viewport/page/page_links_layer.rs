use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use mutool::PageLink;
use std::sync::Arc;

#[derive(IntoElement)]
pub struct PageLinksLayer {
  page_number: usize,
  state: Entity<BookViewerState>,
  links: Option<Arc<Vec<PageLink>>>,
}

impl PageLinksLayer {
  pub fn new(
    page_number: usize, state: Entity<BookViewerState>, links: Option<Arc<Vec<PageLink>>>,
  ) -> Self {
    Self { page_number, state, links }
  }
}

impl RenderOnce for PageLinksLayer {
  fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
    let zoom_factor = self.state.read(cx).zoom_factor;
    let mut link_elements = Vec::new();

    if let Some(links) = &self.links {
      for link in links.iter() {
        let scaled = link.bbox.scaled(zoom_factor);
        let dest_page = link.dest_page;
        let uri = link.uri.clone();
        let state_entity = self.state.clone();

        let link_el = div()
          .absolute()
          .left(px(scaled.x))
          .top(px(scaled.y))
          .w(px(scaled.w))
          .h(px(scaled.h))
          .cursor_pointer()
          .hover(|s| {
            s.border_1().border_color(gpui::blue().opacity(0.6)).bg(gpui::blue().opacity(0.08))
          })
          .on_mouse_down(MouseButton::Left, move |_event: &MouseDownEvent, _window, cx| {
            if let Some(page) = dest_page {
              state_entity.update(cx, |s, cx| {
                s.go_to_page(page);
                cx.notify();
              });
            } else if uri.starts_with("http://") || uri.starts_with("https://") {
              cx.open_url(&uri);
            }
          });

        link_elements.push(link_el);
      }
    }

    div().absolute().inset_0().children(link_elements)
  }
}
