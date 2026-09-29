# MuPDF & GPUI Kit Architecture Documentation

This document series provides a technical specification of the original MuPDF viewer architecture (based on the reference implementations in `other/mupdf.js` and `other/mupdf-webviewer`) and the implementation blueprint for the `libera-reader` desktop application powered by GPUI Kit.

---

## Series Outline

1. [**01. Coordinate Spaces, Scale (Zoom), and HiDPI (DPI)**](01-coordinate-spaces-and-dpi.md)
   - The four coordinate spaces (PDF points, GPUI `px`, physical display pixels, pixmap buffer).
   - Mathematical scaling rules for vector overlays.
   - Two-level raster caching: instant GPU texture scaling + debounced high-DPI background re-rendering.

2. [**02. Page Layer Stack Architecture**](02-page-layer-stack.md)
   - Component hierarchy: `PageCanvas` $\to$ `PageLinksLayer` $\to$ `PageTextLayer` $\to$ `PageShadow`.
   - Z-index rules, transparency, and mouse event routing without link/selection conflicts.
   - Removal of legacy dead code (`page_search_layer.rs`).

3. [**03. Text Layer and Mouse Selection**](03-text-layer-and-selection.md)
   - `PageStructuredText` data layout.
   - Font Metric Mismatch mechanics: why external UI fonts diverge from embedded PDF fonts.
   - MuPDF's SVG glyph stretching technique and its translation into GPUI Kit via `gpui_base::TextSelection`.

4. [**04. Full-Text Search Engine and Navigation**](04-search-engine.md)
   - MuPDF core engine `toStructuredText().search(needle)` and Quad polygons.
   - Sub-second execution performance across 500+ pages.
   - `BookViewerState` state machine: typing isolation (`has_searched`), loading indicator, cyclic `Enter`/`Shift+Enter` traversal.
   - Dynamic theme styling via `cx.theme().primary`.

5. [**05. Document Outline and Hyperlinks**](05-outline-and-links.md)
   - Outline trees (`doc.loadOutline()`) and integration with GPUI Kit's `Tree` component.
   - On-page link resolution (`doc.resolveLink`) and external URI dispatch.
   - Mouse event prioritization to isolate navigation clicks from text drag selection.

6. [**06. Annotations and Text Markups**](06-annotations.md)
   - ISO 32000 annotation types (Highlight, Underline, StrikeOut, FreeText, Stamp, Text Note).
   - WebViewer specification from `mupdf-webviewer/types.d.ts`.
   - Two-phase implementation roadmap: visual reading $\to$ interactive creation, local `native_db` storage, and PDF baking.
