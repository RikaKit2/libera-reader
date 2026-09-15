use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use mutool::PageStructuredText;
use std::sync::Arc;

#[derive(IntoElement)]
pub struct PageSearchLayer {
  page_number: usize,
  state: Entity<BookViewerState>,
  stext: Option<Arc<PageStructuredText>>,
}

impl PageSearchLayer {
  pub fn new(
    page_number: usize, state: Entity<BookViewerState>, stext: Option<Arc<PageStructuredText>>,
  ) -> Self {
    Self { page_number, state, stext }
  }
}

impl RenderOnce for PageSearchLayer {
  fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
    let (zoom_factor, search_query, active_hit_on_this_page) = {
      let s = self.state.read(cx);
      let active_idx = if !s.search_results.is_empty()
        && s.search_results.get(s.current_search_idx).is_some_and(|h| h.page == self.page_number)
      {
        let count_before = s.search_results[..s.current_search_idx]
          .iter()
          .filter(|hit| hit.page == self.page_number)
          .count();
        Some(count_before)
      } else {
        None
      };

      (s.zoom_factor, s.search_query.clone(), active_idx)
    };

    let mut hit_elements = Vec::new();

    if !search_query.trim().is_empty()
      && let Some(stext) = &self.stext
    {
      let query_lower = search_query.to_lowercase();
      let mut hit_idx_on_page = 0;

      for block in &stext.blocks {
        for line in &block.lines {
          let text_lower = line.text.to_lowercase();
          let line_len = line.text.chars().count();
          if line_len == 0 {
            continue;
          }

          let scaled = line.bbox.scaled(zoom_factor);
          let char_width = scaled.w / (line_len as f32);

          let mut start_idx = 0;
          while let Some(found_byte_pos) = text_lower[start_idx..].find(&query_lower) {
            let actual_byte_pos = start_idx + found_byte_pos;
            let char_offset = line.text[..actual_byte_pos].chars().count();
            let query_char_len = search_query.chars().count();

            let hit_x = scaled.x + (char_offset as f32) * char_width;
            let hit_w = (query_char_len as f32) * char_width;

            let is_active = active_hit_on_this_page == Some(hit_idx_on_page);
            hit_idx_on_page += 1;

            let (bg_color, border_color) = if is_active {
              // Vibrant orange highlight for active match under search focus
              (gpui::rgba(0xff9800cc), gpui::rgba(0xff5722ff))
            } else {
              // Soft pink/yellow highlight for all other occurrences on page
              (gpui::rgba(0xffb6c166), gpui::rgba(0xff69b4cc))
            };

            let hit_el = div()
              .absolute()
              .left(px(hit_x))
              .top(px(scaled.y))
              .w(px(hit_w.max(char_width)))
              .h(px(scaled.h))
              .bg(bg_color)
              .border_1()
              .border_color(border_color)
              .rounded(px(2.0));

            hit_elements.push(hit_el);

            start_idx = actual_byte_pos + search_query.len();
            if start_idx >= text_lower.len() {
              break;
            }
          }
        }
      }
    }

    div().absolute().inset_0().children(hit_elements)
  }
}
