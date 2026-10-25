use crate::ui::pages::book_viewer::state::{BookViewerState, PdfTextPos};
use gpui::*;
use gpui_component::ActiveTheme;
use mutool::{BBox, PageStructuredText};
use std::ops::Range;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct PdfWordLayout {
  pub bbox: BBox, // В базовых PDF-пунктах (72 DPI)
  pub text: String,
  pub char_count: usize,
}

#[derive(Clone, Debug)]
pub struct PdfLineLayout {
  pub bbox: BBox, // В базовых PDF-пунктах (72 DPI)
  pub words: Vec<PdfWordLayout>,
}

pub struct PageTextLayer {
  page_number: usize,
  state: Entity<BookViewerState>,
  zoom_factor: f32,
  lines: Arc<Vec<PdfLineLayout>>,
  page_width: f32,
  page_height: f32,
}

impl PageTextLayer {
  pub fn new(
    page_number: usize, state: Entity<BookViewerState>, zoom_factor: f32,
    stext: Option<Arc<PageStructuredText>>, page_size: (f32, f32),
  ) -> Self {
    let (page_width, page_height) = page_size;
    let mut lines = Vec::new();

    if let Some(stext) = stext {
      for block in &stext.blocks {
        for line in &block.lines {
          let mut words = Vec::new();
          if !line.words.is_empty() {
            for w in &line.words {
              let char_count = w.text.chars().count();
              if char_count > 0 {
                words.push(PdfWordLayout { bbox: w.bbox, text: w.text.clone(), char_count });
              }
            }
          } else if !line.text.trim().is_empty() {
            // Fallback, если stext получен без пословной разбивки
            let raw_words: Vec<&str> = line.text.split_whitespace().collect();
            let total_chars: usize = raw_words.iter().map(|w| w.chars().count()).sum::<usize>();
            let spaces = raw_words.len().saturating_sub(1);
            let total_units = (total_chars + spaces).max(1) as f32;
            let unit_w = line.bbox.w / total_units;
            let mut cur_x = line.bbox.x;
            for rw in raw_words {
              let cc = rw.chars().count();
              let ww = unit_w * (cc as f32);
              words.push(PdfWordLayout {
                bbox: BBox::new(cur_x, line.bbox.y, ww, line.bbox.h),
                text: rw.to_string(),
                char_count: cc,
              });
              cur_x += ww + unit_w;
            }
          }

          if !words.is_empty() {
            lines.push(PdfLineLayout { bbox: line.bbox, words });
          }
        }
      }
    }

    Self { page_number, state, zoom_factor, lines: Arc::new(lines), page_width, page_height }
  }

  /// Проверяет, находится ли точка (в PDF-координатах 72 DPI)Directly над каким-либо словом.
  fn is_point_over_text(lines: &[PdfLineLayout], pdf_x: f32, pdf_y: f32) -> bool {
    for line in lines {
      if pdf_y >= line.bbox.y - 2.0
        && pdf_y <= line.bbox.y + line.bbox.h + 2.0
        && let (Some(first), Some(last)) = (line.words.first(), line.words.last())
      {
        let left = first.bbox.x.min(line.bbox.x) - 2.0;
        let right = (last.bbox.x + last.bbox.w).max(line.bbox.x + line.bbox.w) + 2.0;
        if pdf_x >= left && pdf_x <= right {
          return true;
        }
      }
    }
    false
  }

  /// Находит логическую позицию (line_idx, word_idx, char_idx) по PDF-координатам.
  fn hit_test_pos(
    page: usize, lines: &[PdfLineLayout], pdf_x: f32, pdf_y: f32, clamp_to_nearest: bool,
  ) -> Option<PdfTextPos> {
    if lines.is_empty() {
      return None;
    }

    // 1. Ищем строку по вертикали
    let mut best_line_idx = None;
    let mut best_dist = f32::MAX;

    for (idx, line) in lines.iter().enumerate() {
      let top = line.bbox.y;
      let bottom = line.bbox.y + line.bbox.h;
      let v_dist = if pdf_y < top {
        top - pdf_y
      } else if pdf_y > bottom {
        pdf_y - bottom
      } else {
        0.0
      };

      // Учитываем горизонтальную близость для многоколоночной вёрстки на одной высоте
      let h_dist = if pdf_x < line.bbox.x {
        line.bbox.x - pdf_x
      } else if pdf_x > line.bbox.x + line.bbox.w {
        pdf_x - (line.bbox.x + line.bbox.w)
      } else {
        0.0
      };

      let score = v_dist * 4.0 + h_dist * 0.1;
      if score < best_dist && (clamp_to_nearest || (v_dist <= 8.0 && h_dist <= 16.0)) {
        best_dist = score;
        best_line_idx = Some(idx);
      }
    }

    let line_idx = best_line_idx?;
    let line = &lines[line_idx];

    // Если при протяжке курсор ушёл выше самой первой строки или ниже самой последней
    if clamp_to_nearest {
      if line_idx == 0 && pdf_y < line.bbox.y - line.bbox.h * 0.5 {
        return Some(PdfTextPos { page, line_idx: 0, word_idx: 0, char_idx: 0 });
      }
      if line_idx + 1 == lines.len() && pdf_y > line.bbox.y + line.bbox.h * 1.5 {
        let last_w = line.words.len().saturating_sub(1);
        return Some(PdfTextPos {
          page,
          line_idx,
          word_idx: last_w,
          char_idx: line.words[last_w].char_count,
        });
      }
    }

    // 2. Ищем слово и символ внутри строки
    let first_word = &line.words[0];
    if pdf_x <= first_word.bbox.x {
      return Some(PdfTextPos { page, line_idx, word_idx: 0, char_idx: 0 });
    }

    let last_w_idx = line.words.len() - 1;
    let last_word = &line.words[last_w_idx];
    if pdf_x >= last_word.bbox.x + last_word.bbox.w {
      return Some(PdfTextPos {
        page,
        line_idx,
        word_idx: last_w_idx,
        char_idx: last_word.char_count,
      });
    }

    for (w_idx, word) in line.words.iter().enumerate() {
      let w_left = word.bbox.x;
      let w_right = word.bbox.x + word.bbox.w;

      if pdf_x >= w_left && pdf_x <= w_right {
        let frac = ((pdf_x - w_left) / word.bbox.w.max(0.1)).clamp(0.0, 1.0);
        let char_idx = ((frac * word.char_count as f32).round() as usize).min(word.char_count);
        return Some(PdfTextPos { page, line_idx, word_idx: w_idx, char_idx });
      }

      // Если курсор попал в пробел между словом w_idx и w_idx + 1
      if w_idx + 1 < line.words.len() {
        let next_word = &line.words[w_idx + 1];
        if pdf_x > w_right && pdf_x < next_word.bbox.x {
          let mid = (w_right + next_word.bbox.x) * 0.5;
          if pdf_x < mid {
            return Some(PdfTextPos { page, line_idx, word_idx: w_idx, char_idx: word.char_count });
          } else {
            return Some(PdfTextPos { page, line_idx, word_idx: w_idx + 1, char_idx: 0 });
          }
        }
      }
    }

    Some(PdfTextPos { page, line_idx, word_idx: last_w_idx, char_idx: last_word.char_count })
  }

  /// Вычисляет диапазон выделенных символов на конкретной строке этой страницы.
  fn line_selection_span(
    page: usize, line_idx: usize, line: &PdfLineLayout, start: PdfTextPos, end: PdfTextPos,
  ) -> Option<((usize, usize), (usize, usize))> {
    if page < start.page || page > end.page {
      return None;
    }
    if page == start.page && line_idx < start.line_idx {
      return None;
    }
    if page == end.page && line_idx > end.line_idx {
      return None;
    }

    let last_w = line.words.len().saturating_sub(1);
    let start_wc = if page == start.page && line_idx == start.line_idx {
      let w = start.word_idx.min(last_w);
      let c = start.char_idx.min(line.words[w].char_count);
      (w, c)
    } else {
      (0, 0)
    };

    let end_wc = if page == end.page && line_idx == end.line_idx {
      let w = end.word_idx.min(last_w);
      let c = end.char_idx.min(line.words[w].char_count);
      (w, c)
    } else {
      (last_w, line.words[last_w].char_count)
    };

    if start_wc >= end_wc {
      return None;
    }
    Some((start_wc, end_wc))
  }

  /// Возвращает PDF X-координату (72 DPI) для (word_idx, char_idx) внутри строки.
  fn char_x_in_line(line: &PdfLineLayout, word_idx: usize, char_idx: usize) -> f32 {
    let word = &line.words[word_idx];
    let frac = (char_idx as f32 / word.char_count.max(1) as f32).clamp(0.0, 1.0);
    word.bbox.x + word.bbox.w * frac
  }

  /// Собирает точный выделенный текст на этой странице (с пробелами между словами и `\n` между строками).
  fn extract_page_selected_text(
    page: usize, lines: &[PdfLineLayout], start: PdfTextPos, end: PdfTextPos,
  ) -> Option<String> {
    if page < start.page || page > end.page || lines.is_empty() {
      return None;
    }

    let mut page_lines_text = Vec::new();
    for (line_idx, line) in lines.iter().enumerate() {
      let Some(((sw, sc), (ew, ec))) = Self::line_selection_span(page, line_idx, line, start, end)
      else {
        continue;
      };

      let mut line_str = String::new();
      for w_idx in sw..=ew {
        let word = &line.words[w_idx];
        let c_start = if w_idx == sw { sc } else { 0 };
        let c_end = if w_idx == ew { ec } else { word.char_count };

        if c_start < c_end {
          let slice: String = word.text.chars().skip(c_start).take(c_end - c_start).collect();
          if !slice.is_empty() {
            if !line_str.is_empty() {
              line_str.push(' ');
            }
            line_str.push_str(&slice);
          }
        }
      }

      if !line_str.is_empty() {
        page_lines_text.push(line_str);
      }
    }

    if page_lines_text.is_empty() { None } else { Some(page_lines_text.join("\n")) }
  }

  pub fn compute_line_selection_bounds(
    _layout: &gpui::TextLayout, line_text_len: usize, range: Range<usize>,
    line_window_bounds: Bounds<Pixels>,
  ) -> Option<Bounds<Pixels>> {
    if range.is_empty() || line_text_len == 0 {
      return None;
    }
    Some(line_window_bounds)
  }
}

impl IntoElement for PageTextLayer {
  type Element = Self;
  fn into_element(self) -> Self::Element {
    self
  }
}

impl Element for PageTextLayer {
  type RequestLayoutState = ();
  type PrepaintState = Hitbox;

  fn id(&self) -> Option<ElementId> {
    Some(ElementId::NamedInteger("page-text-layer".into(), self.page_number as u64))
  }

  fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
    None
  }

  fn request_layout(
    &mut self, _id: Option<&GlobalElementId>, _inspector_id: Option<&InspectorElementId>,
    window: &mut Window, cx: &mut App,
  ) -> (LayoutId, Self::RequestLayoutState) {
    let mut style = Style::default();
    style.size.width = px(self.page_width).into();
    style.size.height = px(self.page_height).into();
    style.position = gpui::Position::Absolute;
    style.inset = gpui::Edges::all(px(0.0).into());
    // 0 дочерних элементов в Taffy — мгновенный layout без искажения слов!
    let layout_id = window.request_layout(style, [], cx);
    (layout_id, ())
  }

  fn prepaint(
    &mut self, _id: Option<&GlobalElementId>, _inspector_id: Option<&InspectorElementId>,
    bounds: Bounds<Pixels>, _: &mut Self::RequestLayoutState, window: &mut Window, _cx: &mut App,
  ) -> Self::PrepaintState {
    window.insert_hitbox(bounds, HitboxBehavior::Normal)
  }

  fn paint(
    &mut self, _id: Option<&GlobalElementId>, _inspector_id: Option<&InspectorElementId>,
    bounds: Bounds<Pixels>, _: &mut Self::RequestLayoutState, hitbox: &mut Self::PrepaintState,
    window: &mut Window, cx: &mut App,
  ) {
    let page_num = self.page_number;
    let zoom = self.zoom_factor.max(0.01);
    let lines = self.lines.clone();
    let state = self.state.clone();

    // 1. Меняем курсор на текстовый IBeam при наведении на текст
    if hitbox.is_hovered(window) {
      let mouse_pos = window.mouse_position();
      let pdf_x = f32::from(mouse_pos.x - bounds.origin.x) / zoom;
      let pdf_y = f32::from(mouse_pos.y - bounds.origin.y) / zoom;
      if Self::is_point_over_text(&lines, pdf_x, pdf_y) {
        window.set_cursor_style(CursorStyle::IBeam, hitbox);
      }
    }

    // 2. Обработка клика мыши (1 клик — курсор, 2 клика — слово, 3 клика — строка)
    window.on_mouse_event({
      let hitbox = hitbox.clone();
      let lines = lines.clone();
      let state = state.clone();
      move |event: &MouseDownEvent, phase, window, cx| {
        if phase != DispatchPhase::Bubble || event.button != MouseButton::Left {
          return;
        }
        if !hitbox.is_hovered(window) {
          return;
        }

        let pdf_x = f32::from(event.position.x - bounds.origin.x) / zoom;
        let pdf_y = f32::from(event.position.y - bounds.origin.y) / zoom;

        if let Some(pos) = Self::hit_test_pos(page_num, &lines, pdf_x, pdf_y, false) {
          state.update(cx, |s, cx| {
            match event.click_count {
              2 => {
                // Двойной клик: строго выделяем одно слово по его BBox из MuPDF
                let word = &lines[pos.line_idx].words[pos.word_idx];
                let start = PdfTextPos {
                  page: page_num,
                  line_idx: pos.line_idx,
                  word_idx: pos.word_idx,
                  char_idx: 0,
                };
                let end = PdfTextPos {
                  page: page_num,
                  line_idx: pos.line_idx,
                  word_idx: pos.word_idx,
                  char_idx: word.char_count,
                };
                s.pdf_selection.anchor = Some(start);
                s.pdf_selection.focus = Some(end);
                s.pdf_selection.is_dragging = false;
                s.selected_page_texts.clear();
                let txt = Self::extract_page_selected_text(page_num, &lines, start, end);
                s.update_page_selected_text(page_num, txt);
              }
              3 => {
                // Тройной клик: выделяем всю строку целиком
                let line = &lines[pos.line_idx];
                let last_w = line.words.len().saturating_sub(1);
                let start =
                  PdfTextPos { page: page_num, line_idx: pos.line_idx, word_idx: 0, char_idx: 0 };
                let end = PdfTextPos {
                  page: page_num,
                  line_idx: pos.line_idx,
                  word_idx: last_w,
                  char_idx: line.words[last_w].char_count,
                };
                s.pdf_selection.anchor = Some(start);
                s.pdf_selection.focus = Some(end);
                s.pdf_selection.is_dragging = false;
                s.selected_page_texts.clear();
                let txt = Self::extract_page_selected_text(page_num, &lines, start, end);
                s.update_page_selected_text(page_num, txt);
              }
              _ => {
                // Одиночный клик: начало нового выделения
                s.pdf_selection.anchor = Some(pos);
                s.pdf_selection.focus = Some(pos);
                s.pdf_selection.is_dragging = true;
                s.selected_page_texts.clear();
                s.selected_text = None;
              }
            }
            cx.notify();
          });
        } else {
          // Клик по пустому полю страницы сбрасывает выделение
          state.update(cx, |s, cx| {
            if s.pdf_selection.anchor.is_some() {
              s.clear_pdf_selection();
              cx.notify();
            }
          });
        }
      }
    });

    // 3. Протяжка выделения мышью (MouseMove)
    window.on_mouse_event({
      let lines = lines.clone();
      let state = state.clone();
      move |event: &MouseMoveEvent, phase, _window, cx| {
        if phase != DispatchPhase::Bubble {
          return;
        }
        let is_dragging = state.read(cx).pdf_selection.is_dragging;
        if !is_dragging {
          return;
        }
        // Обновляем focus, если курсор находится в пределах этой страницы
        if !bounds.contains(&event.position) {
          return;
        }
        let pdf_x = f32::from(event.position.x - bounds.origin.x) / zoom;
        let pdf_y = f32::from(event.position.y - bounds.origin.y) / zoom;
        if let Some(pos) = Self::hit_test_pos(page_num, &lines, pdf_x, pdf_y, true) {
          state.update(cx, |s, cx| {
            if s.pdf_selection.focus != Some(pos) {
              s.pdf_selection.focus = Some(pos);
              if let Some((start, end)) = s.pdf_selection.ordered_range() {
                let txt = Self::extract_page_selected_text(page_num, &lines, start, end);
                s.update_page_selected_text(page_num, txt);
              }
              cx.notify();
            }
          });
        }
      }
    });

    // 4. Завершение протяжки (MouseUp)
    window.on_mouse_event({
      let state = state.clone();
      move |event: &MouseUpEvent, phase, _window, cx| {
        if phase != DispatchPhase::Bubble || event.button != MouseButton::Left {
          return;
        }
        let is_dragging = state.read(cx).pdf_selection.is_dragging;
        if is_dragging {
          state.update(cx, |s, cx| {
            s.pdf_selection.is_dragging = false;
            #[cfg(target_os = "linux")]
            if let Some(text) = &s.selected_text
              && !text.is_empty()
            {
              cx.write_to_primary(ClipboardItem::new_string(text.clone()));
            }
            cx.notify();
          });
        }
      }
    });

    // 5. Отрисовка результатов поиска MuPDF
    let (exact_page_hits, ordered_selection) = {
      let s = self.state.read(cx);
      let active_idx = s.current_search_idx;
      let hits: Vec<(BBox, bool)> = s
        .search_results
        .iter()
        .enumerate()
        .filter(|(_, hit)| hit.page == page_num)
        .filter_map(|(global_idx, hit)| hit.bbox.map(|bbox| (bbox, global_idx == active_idx)))
        .collect();
      (hits, s.pdf_selection.ordered_range())
    };

    let primary = cx.theme().primary;
    let selection_color = primary.opacity(0.35);

    for (bbox, is_active) in &exact_page_hits {
      let scaled = bbox.scaled(self.zoom_factor);
      let hit_bounds = Bounds::new(
        Point::new(bounds.origin.x + px(scaled.x), bounds.origin.y + px(scaled.y)),
        size(px(scaled.w), px(scaled.h)),
      );
      let (bg_color, border_color) = if *is_active {
        (primary.opacity(0.65), primary)
      } else {
        (primary.opacity(0.20), primary.opacity(0.40))
      };
      window.paint_quad(PaintQuad {
        bounds: hit_bounds,
        background: bg_color.into(),
        corner_radii: Corners::all(px(2.0)),
        border_widths: Edges::all(px(1.0)),
        border_color,
        border_style: BorderStyle::Solid,
      });
    }

    // 6. Отрисовка выделения текста (сплошная плашка на строку вместе с пробелами, как в Firefox)
    if let Some((start, end)) = ordered_selection {
      let page_text = Self::extract_page_selected_text(page_num, &self.lines, start, end);
      if self.state.read(cx).selected_page_texts.get(&page_num) != page_text.as_ref() {
        let state_sync = self.state.clone();
        state_sync.update(cx, |s, _cx| {
          s.update_page_selected_text(page_num, page_text);
        });
      }

      for (line_idx, line) in self.lines.iter().enumerate() {
        let Some(((sw, sc), (ew, ec))) =
          Self::line_selection_span(page_num, line_idx, line, start, end)
        else {
          continue;
        };

        let x0 = Self::char_x_in_line(line, sw, sc) * self.zoom_factor;
        let x1 = Self::char_x_in_line(line, ew, ec) * self.zoom_factor;
        if x1 <= x0 {
          continue;
        }

        let y = line.bbox.y * self.zoom_factor;
        let h = line.bbox.h * self.zoom_factor;

        let highlight_bounds = Bounds::new(
          Point::new(bounds.origin.x + px(x0), bounds.origin.y + px(y)),
          size(px(x1 - x0), px(h)),
        );

        window.paint_quad(PaintQuad {
          bounds: highlight_bounds,
          background: selection_color.into(),
          corner_radii: Corners::all(px(2.0)),
          border_widths: Edges::default(),
          border_color: transparent_black(),
          border_style: BorderStyle::default(),
        });
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use gpui::{point, px, size};
  use mutool::{BBox, FontInfo, TextLine};

  #[core::prelude::v1::test]
  fn test_page_structured_text_scaling() {
    let line = TextLine {
      bbox: BBox::new(50.0, 100.0, 200.0, 16.0),
      font: Some(FontInfo {
        name: Some("Helvetica".into()),
        family: Some("sans-serif".into()),
        weight: None,
        style: None,
        size: 14.0,
      }),
      text: "Sample Line".into(),
      words: Vec::new(),
      x: 50.0,
      y: 110.0,
      wmode: 0,
    };
    let scaled = line.bbox.scaled(1.5);
    assert_eq!(scaled.x, 75.0);
    assert_eq!(scaled.y, 150.0);
    assert_eq!(scaled.w, 300.0);
    assert_eq!(scaled.h, 24.0);
  }

  #[core::prelude::v1::test]
  fn test_compute_line_selection_bounds_full() {
    let line_bounds = Bounds::new(point(px(50.0), px(100.0)), size(px(200.0), px(20.0)));
    let result = PageTextLayer::compute_line_selection_bounds(
      &gpui::TextLayout::default(),
      10,
      0..10,
      line_bounds,
    );
    assert_eq!(result, Some(line_bounds));
  }
}
