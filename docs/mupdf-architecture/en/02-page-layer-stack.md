# 02. Page Layer Stack Architecture

This document describes the visual and interactive layer hierarchy of a rendered document page, Z-index stacking rules, and mouse event routing in GPUI Kit.

---

## 1. Reference Layer Model in MuPDF (`mupdf.js`)

In the reference web implementation (`other/mupdf.js/examples/simple-viewer/viewer.js` and `style.css`), each page is a relative container (`position: relative`) holding absolutely positioned child layers (`position: absolute; left: 0; top: 0; width: 100%; height: 100%`):

```
+-------------------------------------------------------------+
|                      div.page (relative)                    |
|                                                             |
|  [Layer 1] canvas (raster bitmap of the page)               |
|  [Layer 2] svg.text (transparent text for mouse selection)  |
|  [Layer 3] div.link (clickable <a> hyperlink bounds)        |
|  [Layer 4] div.search (search highlight bounding boxes)     |
+-------------------------------------------------------------+
```

### Mouse Event Rules in Reference:
1. **Search Highlight Layer (`div.search > div`)**:
   - `pointer-events: none;` — Clicks and text drag selections pass unobstructed through the highlights to the underlying text.
   - `mix-blend-mode: multiply;` — The highlight color blends naturally with the raster letters without obscuring them.
2. **Hyperlinks Layer (`div.link`)**:
   - On normal hover, displays `cursor: pointer` and handles navigation clicks.
   - While the user is dragging to select text, the `.do-content-select` class is applied:
     ```css
     #pages.do-content-select div.link {
         pointer-events: none;
     }
     ```
     This prevents accidental link navigation when a drag selection path intersects a link bounding box.

---

## 2. Layer Architecture in GPUI Kit (`libera-reader`)

In the GPUI Kit desktop application, the page stack is implemented inside `PageView` (`crates/libera-reader/src/ui/pages/book-viewer/viewport/page/mod.rs`):

```
PageView (div.flex.flex_col.items_center)
│
├── Page Container (div.relative.w(px).h(px))
│   ├── 1. PageCanvas (RenderImage bitmap + dark mode background)
│   ├── 2. PageLinksLayer (internal/external link hitboxes)
│   ├── 3. PageTextLayer (custom Element: transparent text + search hits + drag selection)
│   └── 4. PageShadow (ambient boundary drop shadow)
│
└── 5. PageNumberBadge (page index pill indicator below the sheet)
```

---

## 3. Detailed Component Breakdown

### Layer 1: `PageCanvas` (Raster Bitmap)
- **Responsibility**: Renders the page bitmap decoded from PNG/BGRA via `gpui::img(ImageSource::Render(img))`.
- **Background**: Displays a theme-colored loading placeholder before the first bitmap arrives.
- **Color Inversion**: When `invert_colors` mode is enabled, the image is inverted and the backing quad adopts `cx.theme().background`.
- **Events**: Pure background presentation; does not intercept mouse clicks.

### Layer 2: `PageLinksLayer` (Hyperlinks)
- **Responsibility**: Renders hitboxes from `page.getLinks()`.
- **Link Types**:
  - Internal (`PageLinkDest::Page(n)`): Jumps to a document page by updating `BookViewerState::current_page`.
  - External (`PageLinkDest::Uri(url)`): Opens external URLs via the platform browser launcher (`cx.open_url`).
- **Interactivity**:
  - Displays `CursorStyle::PointingHand` on hover.
  - Activated only on a clean click (mouse down and up at the same coordinate), preventing interference with drag selection.

### Layer 3: `PageTextLayer` (Text Layer, Selection & Search)
- **Responsibility**:
  1. Integrates with GPUI's text selection engine (`gpui_base::TextSelection`).
  2. Paints bounding boxes for full-document search matches (`DocumentSearchMatch`).
  3. Paints mouse selection bounds (`projection.ranges()`).
  4. Hosts transparent text glyphs for mouse cursor hit-testing.
- **Implementation**: Low-level `gpui::Element` implementing `request_layout`, `prepaint` (hitbox insertion via `window.insert_hitbox`), and `paint` (rendering quad highlights via `window.paint_quad`).
- **Styling**:
  - Active hit: `primary.opacity(0.65)` with a solid `primary` border (`1px`, `2px` radius).
  - Passive hits: `primary.opacity(0.20)` with a subtle `primary.opacity(0.40)` border.
  - Rendered beneath the transparent text layer and above the canvas bitmap.

### Layer 4: `PageShadow`
- **Responsibility**: Provides page elevation and page boundary contrast via a border (`cx.theme().border`) and shadow styling.

---

## 4. Legacy Code Cleanup

- **`page_search_layer.rs`**: Previously located at `crates/libera-reader/src/ui/pages/book-viewer/viewport/page/page_search_layer.rs`. It is no longer connected to the `PageView` render tree as all search rendering is consolidated into `PageTextLayer`.
- This file is orphaned dead code and must be removed during implementation to keep the codebase clean.
