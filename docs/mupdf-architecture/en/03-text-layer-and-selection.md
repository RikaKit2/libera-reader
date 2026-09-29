# 03. Text Layer and Mouse Selection

This document describes the structured text data representation in MuPDF, the mechanics behind the Font Metric Mismatch issue, and the architecture for accurate text selection in GPUI Kit.

---

## 1. `PageStructuredText` Data Structure

MuPDF extracts page text with preserved layout geometry via:
```javascript
page.toStructuredText("preserve-spans").asJSON()
```
In Rust, this structure is represented in `crates/mutool/src/stext.rs`:

```rust
pub struct PageStructuredText {
    pub blocks: Vec<TextBlock>,
}

pub struct TextBlock {
    pub block_type: String, // "text" or "image"
    pub bbox: BBox,         // { x, y, w, h } in PDF points (72 DPI)
    pub lines: Vec<TextLine>,
}

pub struct TextLine {
    pub bbox: BBox,
    pub font: Option<FontInfo>, // name, family, weight, style, size
    pub text: String,           // UTF-8 line text
    pub x: f32,
    pub y: f32,
    pub wmode: i32,             // 0 = horizontal text, 1 = vertical text
}
```

Each `TextLine` contains:
- The exact line bounding box `bbox` computed by MuPDF using the font metrics embedded in the PDF.
- The baseline coordinate `y` and start point `x`.
- The font size `font.size` in PDF points.

---

## 2. Root Cause: Font Metric Mismatch (Font Substitution Drift)

### Why metric divergence occurs:
1. **Embedded PDF Font**: Book typography often uses custom subsetted publisher fonts embedded in the PDF (e.g. `BCDHEE+___WRD_EMBED_SUB_43` or `Minion Pro`).
2. **System Interface Font**: GPUI shapes text layout (`window.text_style()`) using the operating system's UI font (such as `Inter`, `Roboto`, or system sans-serif).
3. **Character Width Discrepancies**:
   - Letter "В" in the book font measures $10.2\text{ pt}$, while in `Inter` it is $12.8\text{ pt}$.
   - Letter "ы" in the book font measures $12.1\text{ pt}$, while in `Inter` it is $14.5\text{ pt}$.
   - The space character in the book font measures $3.5\text{ pt}$, while in `Inter` it is $4.2\text{ pt}$.
4. **Outcome**:
   A short phrase ("Вы смогли") measures $61.46\text{ pt}$ ($92.19\text{ px}$ at 150% zoom) in the book's embedded font. The identical string shaped via system `TextLayout` occupies $76.93\text{ pt}$ ($115.4\text{ px}$).
   The accumulated discrepancy of $+23.2\text{ px}$ causes the highlight bounding box to overshoot to the right, covering the inter-word space and the initial letter of the adjacent word.

---

## 3. Reference Solution in `mupdf.js`

In web browsers, custom subsetted fonts embedded in PDFs cannot be injected directly into the document CSSOM for every paragraph.
Consequently, Artifex in `simple-viewer/viewer.js` implemented SVG text stretching:

```javascript
let text = document.createElementNS("http://www.w3.org/2000/svg", "text");
text.setAttribute("x", line.bbox.x * scale + "px");
text.setAttribute("y", line.y * scale + "px");
text.style.fontSize = line.font.size * scale + "px";
text.setAttribute("textLength", line.bbox.w * scale + "px");
text.setAttribute("lengthAdjust", "spacingAndGlyphs");
text.textContent = line.text;
```

### Mechanics:
- `textLength = line.bbox.w * scale` forces the line width to match MuPDF's exact bounding box width.
- `lengthAdjust="spacingAndGlyphs"` instructs the renderer to proportionally scale character spacing and glyph advances so the rendered string spans from `line.bbox.x` to `line.bbox.x + line.bbox.w`.
- The transparent text layer aligns pixel-for-pixel over the rasterized book letters, keeping mouse selection aligned with the visual glyphs.

---

## 4. Implementation in GPUI Kit

Because GPUI Kit does not use an SVG renderer, the text layer integrates through `gpui_base::TextSelection`:

### 4.1. Building `PageTextLineData`
For each `TextLine`:
```rust
let scaled = line.bbox.scaled(zoom_factor);
let font_size = line.font.as_ref().map_or(scaled.h * 0.8, |f| f.size * zoom_factor);

let mut line_style = text_style.clone();
line_style.font_size = px(font_size).into();
line_style.line_height = px(scaled.h).into();
line_style.color = gpui::transparent_black(); // Completely transparent

let runs = vec![line_style.to_run(line.text.len())];
let shared_text = SharedString::from(line.text.clone());
let styled_text = StyledText::new(shared_text.clone()).with_runs(runs);

let line_bounds = Bounds::new(
    Point::new(px(scaled.x), px(scaled.y)),
    size(px(scaled.w), px(scaled.h)),
);
```

### 4.2. Computing Sub-range Bounds (`compute_line_selection_bounds`)
For partial mouse drag selections:
- Relative offsets are sampled using `layout.position_for_index(start)` and `layout.position_for_index(end)`.
- Offsets are clamped to the physical line bounds:
  ```rust
  let left_bound = line_window_bounds.origin.x;
  let right_bound = line_window_bounds.right();
  let x_start = layout.position_for_index(range.start)
      .map(|p| p.x.max(left_bound).min(right_bound))
      .unwrap_or(left_bound);
  ```
- For search matches from `search_document_text`, text layout estimations are bypassed entirely: coordinates are consumed directly from MuPDF `hit.bbox.scaled(zoom_factor)`.

### 4.3. Registration with GPUI Text Selection Manager
During `prepaint`:
```rust
self.selection_handle.register(
    TextSelectionRegistration::new(page_hitbox.clone(), bounds)
        .with_document_order(self.page_number as u64)
        .with_text_bounds(text_bounds),
    window,
    cx,
);
```
During `paint`:
```rust
let projection = self.selection_handle.update_runs(&runs, cx);
```
This enables:
- Multi-line and cross-page continuous mouse drag selection.
- Copying pure UTF-8 document text to the clipboard (`Ctrl+C` / `Cmd+C`).
- Selection highlight rendering using the active theme: `cx.theme().primary.opacity(0.35)`.
