use gpui::*;
use gpui_base::{TextSelection, TextSelectionHandle, TextSelectionRegistration, TextSelectionRun};
use mutool::PageStructuredText;
use std::ops::Range;
use std::sync::Arc;

pub struct PageTextLineData {
  pub bounds: Bounds<Pixels>,
  pub text: SharedString,
  pub styled_text: StyledText,
  pub document_order: u64,
}

pub struct PageTextLayer {
  page_number: usize,
  zoom_factor: f32,
  selection_handle: TextSelectionHandle,
  lines: Vec<PageTextLineData>,
  page_width: f32,
  page_height: f32,
}

impl PageTextLayer {
  pub fn new(
    page_number: usize, zoom_factor: f32, selection_handle: TextSelectionHandle,
    stext: Option<Arc<PageStructuredText>>, page_width: f32, page_height: f32, window: &mut Window,
  ) -> Self {
    let mut lines = Vec::new();

    if let Some(stext) = stext {
      let mut text_style = window.text_style();
      text_style.color = gpui::transparent_black();

      let mut global_line_idx = 0;
      for block in &stext.blocks {
        for line in &block.lines {
          let scaled = line.bbox.scaled(zoom_factor);
          let font_size = line.font.as_ref().map_or(scaled.h * 0.8, |f| f.size * zoom_factor);
          let mut line_style = text_style.clone();
          line_style.font_size = px(font_size).into();
          line_style.line_height = px(scaled.h).into();

          let runs = vec![line_style.to_run(line.text.len())];
          let shared_text = SharedString::from(line.text.clone());
          let styled_text = StyledText::new(shared_text.clone()).with_runs(runs);

          let line_bounds =
            Bounds::new(Point::new(px(scaled.x), px(scaled.y)), size(px(scaled.w), px(scaled.h)));

          lines.push(PageTextLineData {
            bounds: line_bounds,
            text: shared_text,
            styled_text,
            document_order: global_line_idx,
          });
          global_line_idx += 1;
        }
      }
    }

    Self { page_number, zoom_factor, selection_handle, lines, page_width, page_height }
  }

  pub fn compute_line_selection_bounds(
    layout: &gpui::TextLayout, line_text_len: usize, range: Range<usize>,
    line_window_bounds: Bounds<Pixels>,
  ) -> Option<Bounds<Pixels>> {
    if range.is_empty() || line_text_len == 0 {
      return None;
    }

    let left_bound = line_window_bounds.origin.x;
    let right_bound = line_window_bounds.right();

    let x_start = if range.start == 0 {
      left_bound
    } else {
      layout
        .position_for_index(range.start)
        .map(|p| p.x.max(left_bound).min(right_bound))
        .unwrap_or(left_bound)
    };

    let x_end = if range.end >= line_text_len {
      right_bound
    } else {
      layout
        .position_for_index(range.end)
        .map(|p| p.x.max(left_bound).min(right_bound))
        .unwrap_or(right_bound)
    };

    let min_x = x_start.min(x_end);
    let max_x = x_start.max(x_end);

    if min_x >= max_x {
      return None;
    }

    Some(Bounds::new(
      Point::new(min_x, line_window_bounds.origin.y),
      size(max_x - min_x, line_window_bounds.size.height),
    ))
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
    &mut self, id: Option<&GlobalElementId>, inspector_id: Option<&InspectorElementId>,
    window: &mut Window, cx: &mut App,
  ) -> (LayoutId, Self::RequestLayoutState) {
    let mut style = Style::default();
    style.size.width = px(self.page_width).into();
    style.size.height = px(self.page_height).into();
    style.position = gpui::Position::Absolute;
    style.inset = gpui::Edges::all(px(0.0).into());

    let mut child_layout_ids = Vec::with_capacity(self.lines.len());
    for line in &mut self.lines {
      let (child_layout_id, _) = line.styled_text.request_layout(id, inspector_id, window, cx);
      child_layout_ids.push(child_layout_id);
    }

    let layout_id = window.request_layout(style, child_layout_ids, cx);
    (layout_id, ())
  }

  fn prepaint(
    &mut self, id: Option<&GlobalElementId>, inspector_id: Option<&InspectorElementId>,
    bounds: Bounds<Pixels>, _: &mut Self::RequestLayoutState, window: &mut Window, cx: &mut App,
  ) -> Self::PrepaintState {
    let page_hitbox = window.insert_hitbox(bounds, HitboxBehavior::Normal);
    let mut text_bounds = Vec::with_capacity(self.lines.len());

    for line in &mut self.lines {
      let line_window_bounds = Bounds::new(
        Point::new(bounds.origin.x + line.bounds.origin.x, bounds.origin.y + line.bounds.origin.y),
        line.bounds.size,
      );
      line.styled_text.prepaint(id, inspector_id, line_window_bounds, &mut (), window, cx);
      text_bounds.push(line_window_bounds);
    }

    self.selection_handle.register(
      TextSelectionRegistration::new(page_hitbox.clone(), bounds)
        .with_document_order(self.page_number as u64)
        .with_text_bounds(text_bounds),
      window,
      cx,
    );

    page_hitbox
  }

  fn paint(
    &mut self, id: Option<&GlobalElementId>, inspector_id: Option<&InspectorElementId>,
    bounds: Bounds<Pixels>, _: &mut Self::RequestLayoutState, _: &mut Self::PrepaintState,
    window: &mut Window, cx: &mut App,
  ) {
    let mut runs = Vec::with_capacity(self.lines.len());

    for line in &self.lines {
      let line_window_bounds = Bounds::new(
        Point::new(bounds.origin.x + line.bounds.origin.x, bounds.origin.y + line.bounds.origin.y),
        line.bounds.size,
      );
      let layout = line.styled_text.layout().clone();
      runs.push(
        TextSelectionRun::new(line.text.clone(), layout, line_window_bounds)
          .with_document_order(line.document_order),
      );
    }

    let selected_text_before = TextSelection::selected_text(window, cx);
    let projection = self.selection_handle.update_runs(&runs, cx);
    if selected_text_before != TextSelection::selected_text(window, cx) {
      window.refresh();
    }

    let highlight_color = gpui::hsla(0.58, 0.85, 0.62, 0.35);

    for (line_idx, line) in self.lines.iter_mut().enumerate() {
      let line_window_bounds = Bounds::new(
        Point::new(bounds.origin.x + line.bounds.origin.x, bounds.origin.y + line.bounds.origin.y),
        line.bounds.size,
      );
      let layout = line.styled_text.layout().clone();

      if let Some(Some(range)) = projection.ranges().get(line_idx)
        && let Some(highlight_bounds) = Self::compute_line_selection_bounds(
          &layout,
          line.text.len(),
          range.clone(),
          line_window_bounds,
        )
      {
        window.paint_quad(PaintQuad {
          bounds: highlight_bounds,
          background: highlight_color.into(),
          corner_radii: Corners::default(),
          border_widths: Edges::default(),
          border_color: transparent_black(),
          border_style: BorderStyle::default(),
        });
      }

      line.styled_text.paint(id, inspector_id, line_window_bounds, &mut (), &mut (), window, cx);
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
    // For full range (0..10), it covers left_bound to right_bound without layout
    let result = PageTextLayer::compute_line_selection_bounds(
      &gpui::TextLayout::default(),
      10,
      0..10,
      line_bounds,
    );

    assert_eq!(result, Some(line_bounds));
  }
}
