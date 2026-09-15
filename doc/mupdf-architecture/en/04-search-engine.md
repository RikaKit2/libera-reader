# 04. Full-Text Search Engine and Navigation

This document details the MuPDF document search algorithm, Quad polygon parsing, the search state machine, and highlight rendering in GPUI Kit.

---

## 1. Search via MuPDF Core Engine (`toStructuredText().search`)

For full-text searching in PDF/EPUB documents, MuPDF provides:
```javascript
var hits = page.toStructuredText().search(needle);
```

### Advantages of `toStructuredText().search`:
- Evaluates text using internal font encoding and CMap tables.
- Built-in Unicode normalization and case-insensitivity.
- Returns matches as arrays of **Quad** polygons (quadrilaterals).

### Quad Polygon Format:
Each Quad contains 8 coordinates `[ulx, uly, urx, ury, llx, lly, lrx, lry]`:
- `ul`: Upper Left
- `ur`: Upper Right
- `ll`: Lower Left
- `lr`: Lower Right

In the search script (`crates/mutool/src/search.rs`), Quads are transformed into `BBox` rectangles:
```javascript
var x0 = Math.min(q[0], q[2], q[4], q[6]);
var y0 = Math.min(q[1], q[3], q[5], q[7]);
var x1 = Math.max(q[0], q[2], q[4], q[6]);
var y1 = Math.max(q[1], q[3], q[5], q[7]);
allHits.push({
    page: p + 1,
    bbox: { x: x0, y: y0, w: x1 - x0, h: y1 - y0 }
});
```
All coordinates are produced in **PDF points** ($72\text{ DPI}$).

---

## 2. Execution Performance

Running a full-document search across a 586-page book (`Всего шесть чисел...`) via `mutool run` takes **~0.8 seconds**.
This sub-second latency enables immediate on-demand execution on `Enter` without requiring background SQLite or Lucene indexing.

---

## 3. Search State Machine (`BookViewerState`)

To eliminate misleading status messages (such as displaying "Not found" while the user is still typing), the state model decouples query entry from executed search results:

```rust
pub struct BookViewerState {
    /// Active search string in the input field
    pub search_query: String,
    /// Collected matches across the entire document
    pub search_results: Vec<DocumentSearchMatch>,
    /// Index of current active match (0..search_results.len())
    pub current_search_idx: usize,
    /// Whether a full-document search has been triggered
    pub has_searched: bool,
    /// Whether an asynchronous search is currently running
    pub is_searching: bool,
}
```

### State Transitions:
```
[Empty] 
   │  (user types query characters)
   ▼
[Typing] ────────── has_searched = false ───► Counter is empty (no false "Not found")
   │  (Enter key / click "Find")
   ▼
[Async Search] ──── is_searching = true ────► Spinner / loading indicator
   │  (mutool process settles)
   ▼
[Results Ready] ─── has_searched = true
   ├── If matches > 0  ──► "1 of 42", active hit centered on screen
   └── If matches == 0 ──► "Not found"
```

---

## 4. Match Traversal (Next / Previous)

Cyclic wrapping traversal:
- **Next (`Enter` / `>` button / `F3`)**:
  ```rust
  if !state.search_results.is_empty() {
      state.current_search_idx = (state.current_search_idx + 1) % state.search_results.len();
      let target_page = state.search_results[state.current_search_idx].page;
      state.scroll_to_page(target_page);
  }
  ```
- **Previous (`Shift+Enter` / `<` button / `Shift+F3`)**:
  ```rust
  if !state.search_results.is_empty() {
      state.current_search_idx = (state.current_search_idx + state.search_results.len() - 1) % state.search_results.len();
      let target_page = state.search_results[state.current_search_idx].page;
      state.scroll_to_page(target_page);
  }
  ```

---

## 5. Highlight Rendering in `PageTextLayer`

All highlight rendering is handled inside `PageTextLayer::paint`:

1. **Scaling**:
   ```rust
   let scaled = bbox.scaled(self.zoom_factor);
   let hit_bounds = Bounds::new(
       Point::new(bounds.origin.x + px(scaled.x), bounds.origin.y + px(scaled.y)),
       size(px(scaled.w), px(scaled.h)),
   );
   ```
2. **Dynamic Theme Tokens (`cx.theme().primary`)**:
   - **Active match** (`is_active == true`):
     - Fill: `primary.opacity(0.65)`
     - Border: `primary` (100% opaque, `1px`, `2px` corner radius)
   - **Passive page matches**:
     - Fill: `primary.opacity(0.20)`
     - Border: `primary.opacity(0.40)`
3. **Hardware Quad Painting**:
   Highlights are drawn into the window buffer via `window.paint_quad`. Because they sit below the transparent text layer, users can still select and copy the highlighted text with a mouse drag.
