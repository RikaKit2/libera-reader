# 05. Document Outline and Hyperlinks

This document details the retrieval and rendering of document outlines (TOC / Bookmarks) and internal/external page hyperlinks.

---

## 1. Document Outline (Table of Contents / Bookmarks)

### 1.1. Extraction via MuPDF
In MuPDF, document outline trees are loaded using:
```javascript
let outline = doc.loadOutline();
```
Each node in the outline tree contains:
- `title`: Section or chapter title.
- `page`: Target page index (0-indexed).
- `uri`: Optional external URI.
- `down`: Child sections (nested outline).
- `next`: Sibling section at the same depth.

Represented in Rust in `crates/mutool/src/outline.rs`:
```rust
pub struct OutlineItem {
    pub title: String,
    pub page: Option<usize>,
    pub uri: Option<String>,
    pub children: Vec<OutlineItem>,
}
```

### 1.2. Presentation in the GPUI Kit Sidebar
- Displayed within the sidebar tab `SidebarTab::Outline` (`crates/libera-reader/src/ui/pages/book-viewer/sidebar/tabs/outline.rs`).
- Powered by GPUI Kit's `Tree` component (`gpui_kit::component::tree::{Tree, TreeState, TreeItem}`) or a tailored hierarchical list.
- Item click action:
  ```rust
  if let Some(target_page) = item.page {
      state.update(cx, |s, cx| {
          s.scroll_to_page(target_page);
          cx.notify();
      });
  }
  ```

---

## 2. On-Page Hyperlinks

### 2.1. Extraction via MuPDF
Page links are retrieved via `page.getLinks()`:
```javascript
let links = page.getLinks();
links.map((link) => {
    const [ x0, y0, x1, y1 ] = link.getBounds();
    let href = link.isExternal() ? link.getURI() : `#page${doc.resolveLink(link) + 1}`;
    return { x: x0, y: y0, w: x1 - x0, h: y1 - y0, href };
});
```

Represented in Rust in `crates/mutool/src/links.rs`:
```rust
pub struct PageLink {
    pub bbox: BBox,              // Coordinates in PDF points (72 DPI)
    pub dest_page: Option<usize>, // 1-indexed target page for document jumps
    pub uri: Option<String>,      // Target URL for external web links
}
```

### 2.2. Rendering in `PageLinksLayer`
- Each link is positioned over `PageCanvas` using `bbox.scaled(zoom_factor)`.
- Handled in `PageLinksLayer` (`crates/libera-reader/src/ui/pages/book-viewer/viewport/page/page_links_layer.rs`):
  - Changes cursor to `CursorStyle::PointingHand` on hover.
  - Highlights with `cx.theme().primary.opacity(0.35)` on hover.
  - Internal link click:
    ```rust
    if let Some(dest) = dest_page {
        state.update(cx, |s, cx| {
            s.scroll_to_page(dest);
            cx.notify();
        });
    }
    ```
  - External link click: Invokes the platform default browser via `cx.open_url(&uri)`.

### 2.3. Event Isolation from Text Selection
- To ensure links do not intercept or abort text drag selections, `PageLinksLayer` fires strictly on clean clicks where pointer movement stays within threshold, delegating drag gestures to `TextSelection`.
