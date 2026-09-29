# MuPDF In-Document Search & Dynamic DPI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix the in-document search functionality by eliminating font metric drift, removing dead legacy search code, implementing dynamic zoom/DPI-aware raster page rendering, and adding smooth viewport centering for search matches.

**Architecture:** Consolidate all search highlight and selection rendering inside `PageTextLayer` using MuPDF's exact bounding boxes (`BBox`). Compute dynamic rasterization DPI based on zoom factor and window scale factor with two-level caching (immediate GPU scale + debounced background high-DPI re-render). Clean up orphaned `page_search_layer.rs` and wire cyclic search traversal with viewport auto-scrolling to active matches.

**Tech Stack:** Rust, GPUI (`gpui`, `gpui-base`, `gpui-component`), `mutool` CLI / MuPDF, `parking_lot`, `lru`.

**Spec:** `docs/mupdf-architecture/en/` and `docs/mupdf-architecture/ru/`:
- `01-coordinate-spaces-and-dpi.md`
- `02-page-layer-stack.md`
- `03-text-layer-and-selection.md`
- `04-search-engine.md`

## Global Constraints

- Never invent an API: check GPUI Kit signatures against local docs in `~/.omp/agent/docs/gpui-kit/`.
- No git commits without explicit user request, and always sign with `-S`.
- All code comments and git commit messages must be strictly in English (Conventional Commits).
- Dynamic theme colors only: use `cx.theme().primary` tokens, never hardcoded hex/rgba.
- Fast verification: run targeted tests (`cargo test -p libera-reader --lib -- <filter>` and `cargo test -p mutool --lib`) rather than full workspace builds when intermediate verification is needed.

---

### Task 1: Remove Dead Legacy Code (`page_search_layer.rs`)

**Files:**
- Delete: `crates/libera-reader/src/ui/pages/book-viewer/viewport/page/page_search_layer.rs`
- Modify: `crates/libera-reader/src/ui/pages/book-viewer/viewport/page/mod.rs:1-40`

**Interfaces:**
- Consumes: None
- Produces: Clean `page` module without dead references to `page_search_layer`

- [ ] **Step 1: Check references to `page_search_layer` in `crates/libera-reader`**

Ensure no other file imports `PageSearchLayer` or `page_search_layer`.
Command: verify with LSP or grep that `page_search_layer` only exists in `page/mod.rs`.

- [ ] **Step 2: Remove `pub mod page_search_layer;` from `page/mod.rs`**

Edit `crates/libera-reader/src/ui/pages/book-viewer/viewport/page/mod.rs` to delete:
```rust
pub mod page_search_layer;
pub use page_search_layer::PageSearchLayer;
```

- [ ] **Step 3: Delete the file `page_search_layer.rs`**

Delete `crates/libera-reader/src/ui/pages/book-viewer/viewport/page/page_search_layer.rs`.

- [ ] **Step 4: Verify that `libera-reader` compiles and tests pass**

Run: `cargo test -p libera-reader --lib -- viewport::page`
Expected: PASS

---

### Task 2: Implement Dynamic DPI Calculation in Book Page Loader

**Files:**
- Modify: `crates/libera-reader/src/ui/pages/book-viewer/constants.rs:240-250`
- Modify: `crates/libera-reader/src/ui/pages/book-viewer/loader.rs:15-35`
- Modify: `crates/libera-reader/src/ui/pages/book-viewer/cache.rs:1-80`
- Modify: `crates/libera-reader/src/ui/pages/book-viewer/viewport/scroll_container.rs:160-180`
- Modify: `crates/libera-reader/src/ui/pages/book-viewer/viewport/paged_container.rs:80-95`
- Test: `crates/libera-reader/src/ui/pages/book-viewer/loader.rs` (unit tests)

**Interfaces:**
- Consumes: `window.scale_factor()`, `state.zoom_factor`, `mutool::render_page_to_png_bytes(path, page, dpi)`
- Produces: `compute_target_dpi(zoom_factor: f32, scale_factor: f32) -> u32`

- [ ] **Step 1: Write unit test for `compute_target_dpi`**

Add test in `crates/libera-reader/src/ui/pages/book-viewer/loader.rs`:
```rust
#[test]
fn test_compute_target_dpi() {
    assert_eq!(compute_target_dpi(1.0, 1.0), 72);
    assert_eq!(compute_target_dpi(1.5, 1.0), 108);
    assert_eq!(compute_target_dpi(2.0, 1.0), 144);
    assert_eq!(compute_target_dpi(1.0, 2.0), 144);
    assert_eq!(compute_target_dpi(1.5, 2.0), 216);
    assert_eq!(compute_target_dpi(0.1, 1.0), 36); // Clamped to minimum 36 DPI
    assert_eq!(compute_target_dpi(5.0, 2.0), 300); // Clamped to maximum 300 DPI
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p libera-reader --lib -- test_compute_target_dpi`
Expected: FAIL with "cannot find function `compute_target_dpi`"

- [ ] **Step 3: Implement `compute_target_dpi`**

In `crates/libera-reader/src/ui/pages/book-viewer/loader.rs`:
```rust
pub const MIN_RENDER_DPI: u32 = 36;
pub const MAX_RENDER_DPI: u32 = 300;
pub const BASE_PDF_DPI: f32 = 72.0;

pub fn compute_target_dpi(zoom_factor: f32, scale_factor: f32) -> u32 {
    let calculated = (BASE_PDF_DPI * zoom_factor * scale_factor).round() as u32;
    calculated.clamp(MIN_RENDER_DPI, MAX_RENDER_DPI)
}
```

- [ ] **Step 4: Update `PageLoadRequest` to carry computed DPI**

In `scroll_container.rs` and `paged_container.rs`, when spawning `PageLoadRequest`:
```rust
let scale_factor = window.scale_factor();
let target_dpi = compute_target_dpi(zoom_factor, scale_factor);
loader_sender.push(PageLoadRequest {
    page: page_num,
    book_path: book_path.clone(),
    dpi: target_dpi,
});
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test -p libera-reader --lib -- test_compute_target_dpi`
Expected: PASS

---

### Task 3: Fix Live Search Preview in `PageTextLayer` to Eliminate Font Drift

**Files:**
- Modify: `crates/libera-reader/src/ui/pages/book-viewer/viewport/page/page_text_layer.rs:250-290`
- Test: `crates/libera-reader/src/ui/pages/book-viewer/viewport/page/page_text_layer.rs` (tests)

**Interfaces:**
- Consumes: `exact_page_hits: Vec<(BBox, bool)>`, `stext: Option<Arc<PageStructuredText>>`
- Produces: Pixel-accurate highlight bounding boxes without font substitution drift

- [ ] **Step 1: Write unit test for sub-word highlight computation from MuPDF `stext`**

Add unit test in `page_text_layer.rs`:
```rust
#[test]
fn test_stext_character_highlight_bounds() {
    let line_bbox = BBox::new(50.0, 100.0, 100.0, 20.0);
    let text = "Hello World";
    // Target "World": index 6..11
    let bounds = compute_proportional_match_bounds(line_bbox, text, 6..11, 1.5);
    assert!(bounds.is_some());
    let b = bounds.unwrap();
    // Origin x must start proportionally along the line
    assert!(b.origin.x > px(line_bbox.x * 1.5));
    assert!(b.size.width > px(0.0));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p libera-reader --lib -- test_stext_character_highlight_bounds`
Expected: FAIL with "cannot find function `compute_proportional_match_bounds`"

- [ ] **Step 3: Implement `compute_proportional_match_bounds`**

In `crates/libera-reader/src/ui/pages/book-viewer/viewport/page/page_text_layer.rs`:
```rust
pub fn compute_proportional_match_bounds(
    line_bbox: BBox,
    text: &str,
    char_range: Range<usize>,
    zoom_factor: f32,
) -> Option<Bounds<Pixels>> {
    let total_chars = text.chars().count();
    if total_chars == 0 || char_range.is_empty() || char_range.end > total_chars {
        return None;
    }

    let scaled = line_bbox.scaled(zoom_factor);
    let char_width = scaled.w / (total_chars as f32);

    let x = scaled.x + (char_range.start as f32) * char_width;
    let w = ((char_range.end - char_range.start) as f32) * char_width;

    Some(Bounds::new(
        Point::new(px(x), px(scaled.y)),
        size(px(w), px(scaled.h)),
    ))
}
```

- [ ] **Step 4: Update live preview highlight in `paint`**

In `page_text_layer.rs`:
Replace the fallback system `TextLayout` calculation with proportional line geometry derived from MuPDF's `line.bbox`, or display live matches only when structured line text matches, styled via `cx.theme().primary.opacity(0.25)` with `primary.opacity(0.50)` border.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p libera-reader --lib -- test_stext_character_highlight_bounds`
Expected: PASS

---

### Task 4: Enhance Search Traversal and Auto-Scroll to Active Match

**Files:**
- Modify: `crates/libera-reader/src/ui/pages/book-viewer/state/mod.rs:200-260`
- Modify: `crates/libera-reader/src/ui/pages/book-viewer/header/search_bar/search_input.rs:40-75`
- Modify: `crates/libera-reader/src/ui/pages/book-viewer/header/search_bar/search_counter.rs:25-50`
- Modify: `crates/libera-reader/src/ui/pages/book-viewer/viewport/scroll_container.rs:180-220`
- Test: `crates/libera-reader/src/ui/pages/book-viewer/state/mod.rs` (search tests)

**Interfaces:**
- Consumes: `state.search_results`, `state.current_search_idx`, `state.has_searched`
- Produces: `next_search_match()`, `prev_search_match()`, active match tracking with auto-scroll

- [ ] **Step 1: Write test for cyclic search traversal in `BookViewerState`**

In `crates/libera-reader/src/ui/pages/book-viewer/state/mod.rs`:
```rust
#[test]
fn test_cyclic_search_traversal() {
    let mut state = BookViewerState::new();
    state.search_results = vec![
        DocumentSearchMatch::new(2, BBox::default(), "match 1"),
        DocumentSearchMatch::new(5, BBox::default(), "match 2"),
        DocumentSearchMatch::new(10, BBox::default(), "match 3"),
    ];
    state.has_searched = true;
    state.current_search_idx = 0;

    state.next_search_match();
    assert_eq!(state.current_search_idx, 1);
    assert_eq!(state.current_page, 5);

    state.next_search_match();
    assert_eq!(state.current_search_idx, 2);
    assert_eq!(state.current_page, 10);

    // Wrap around to start
    state.next_search_match();
    assert_eq!(state.current_search_idx, 0);
    assert_eq!(state.current_page, 2);

    // Wrap around to end
    state.prev_search_match();
    assert_eq!(state.current_search_idx, 2);
    assert_eq!(state.current_page, 10);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p libera-reader --lib -- test_cyclic_search_traversal`
Expected: FAIL if methods are missing or behavior differs

- [ ] **Step 3: Implement `next_search_match` and `prev_search_match`**

In `BookViewerState`:
```rust
pub fn next_search_match(&mut self) {
    if self.search_results.is_empty() {
        return;
    }
    self.current_search_idx = (self.current_search_idx + 1) % self.search_results.len();
    let target_page = self.search_results[self.current_search_idx].page;
    self.scroll_to_page(target_page);
}

pub fn prev_search_match(&mut self) {
    if self.search_results.is_empty() {
        return;
    }
    let len = self.search_results.len();
    self.current_search_idx = (self.current_search_idx + len - 1) % len;
    let target_page = self.search_results[self.current_search_idx].page;
    self.scroll_to_page(target_page);
}
```

- [ ] **Step 4: Wire `Enter` and `Shift+Enter` in `search_input.rs`**

Ensure `on_key_down` triggers `next_search_match` on `Enter` and `prev_search_match` on `Shift+Enter`.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p libera-reader --lib -- test_cyclic_search_traversal`
Expected: PASS

---

### Task 5: Verification and Final Validation

**Files:**
- All touched files in `crates/libera-reader` and `crates/mutool`

- [ ] **Step 1: Run all unit tests in `mutool`**

Run: `cargo test -p mutool --lib`
Expected: PASS (all tests pass)

- [ ] **Step 2: Run all unit tests in `libera-reader`**

Run: `cargo test -p libera-reader --lib`
Expected: PASS (all tests pass)
