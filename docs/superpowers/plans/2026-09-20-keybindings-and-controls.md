# Keybindings System, Zoom Controls & Settings Editor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement a declarative, customizable keybindings system across the entire reader, add a Keybindings Editor in Settings with conflict detection and persistence, fix FitWidth/FitPage zoom calculations with Ctrl+Wheel zoom, and virtualize the Thumbnails sidebar tab.

**Architecture:** Use GPUI's declarative `actions!` macro and `KeyBinding` engine scoped via `key_context("BookViewer")`. Store user keybindings in a `KeybindingsConfig` model persisted in `native_db` under `SETTINGS`. Build an interactive Keybindings Settings tab with categorized action lists, key combination recorder, conflict detection, and reset to defaults. Implement dynamic viewport dimension calculations for `FitWidth`/`FitPage` zoom modes, capture mouse wheel zoom with Ctrl, and virtualize `ThumbnailsView` using `v_virtual_list`.

**Tech Stack:** Rust, GPUI (`gpui`, `gpui-base`, `gpui-component` / `gpui-kit`), `native_db`, `serde`, `rust-i18n`.

**Spec:**
- `docs/ROADMAP.md` §1.1, §1.2, §1.3, §1.4
- `docs/mupdf-architecture/en/01-coordinate-spaces-and-dpi.md`
- `docs/mupdf-architecture/en/02-page-layer-stack.md`
- Local skill reference: `skill://gpui-kit/references/gpui/action.md`

## Global Constraints

- Follow GPUI Kit coding conventions from `skill://gpui-kit`: no invented APIs, semantic theme tokens `cx.theme().primary` only.
- Strict English for all code symbols, doc-comments, and commit messages (Conventional Commits).
- No git commits without explicit user request, always signed with `-S`.
- Fast, targeted verification only: do not run full workspace builds; verify with targeted tests.

---

### Task 1: Declarative GPUI Actions & Default Keybindings

**Files:**
- Create: `crates/libera-reader/src/ui/pages/book-viewer/actions.rs`
- Modify: `crates/libera-reader/src/ui/pages/book-viewer/mod.rs`
- Modify: `crates/libera-reader/src/ui/pages/book-viewer/header/header_bar.rs`

**Interfaces:**
- Consumes: `BookViewerState`, GPUI `actions!` macro, `KeyBinding`
- Produces: `pub mod actions;` with action types and default bindings generator `default_keybindings()`

- [ ] **Step 1: Define actions in `actions.rs`**

Create `crates/libera-reader/src/ui/pages/book-viewer/actions.rs`:
```rust
use gpui::*;

actions!(
  book_viewer,
  [
    NextPage,
    PrevPage,
    FirstPage,
    LastPage,
    ZoomIn,
    ZoomOut,
    ResetZoom,
    FitWidth,
    FitPage,
    ToggleSearch,
    NextSearchMatch,
    PrevSearchMatch,
    CloseSearch,
    ToggleFullscreen,
    ToggleInvertColors,
    ToggleSidebar,
    ToggleBookmarks,
    ToggleOutline,
    ToggleThumbnails,
    ToggleTts,
    TtsPlayPause,
    TtsNext,
    TtsPrev,
    ExitViewer,
  ]
);

pub const BOOK_VIEWER_CONTEXT: &str = "BookViewer";

pub fn default_keybindings() -> Vec<KeyBinding> {
  vec![
    // Navigation
    KeyBinding::new("right", NextPage, Some(BOOK_VIEWER_CONTEXT)),
    KeyBinding::new("pageup", PrevPage, Some(BOOK_VIEWER_CONTEXT)),
    KeyBinding::new("pagedown", NextPage, Some(BOOK_VIEWER_CONTEXT)),
    KeyBinding::new("left", PrevPage, Some(BOOK_VIEWER_CONTEXT)),
    KeyBinding::new("home", FirstPage, Some(BOOK_VIEWER_CONTEXT)),
    KeyBinding::new("end", LastPage, Some(BOOK_VIEWER_CONTEXT)),
    KeyBinding::new("space", NextPage, Some(BOOK_VIEWER_CONTEXT)),
    KeyBinding::new("shift-space", PrevPage, Some(BOOK_VIEWER_CONTEXT)),

    // Search
    KeyBinding::new("ctrl-f", ToggleSearch, Some(BOOK_VIEWER_CONTEXT)),
    KeyBinding::new("f3", NextSearchMatch, Some(BOOK_VIEWER_CONTEXT)),
    KeyBinding::new("shift-f3", PrevSearchMatch, Some(BOOK_VIEWER_CONTEXT)),
    KeyBinding::new("escape", CloseSearch, Some(BOOK_VIEWER_CONTEXT)),

    // Zoom
    KeyBinding::new("ctrl-=", ZoomIn, Some(BOOK_VIEWER_CONTEXT)),
    KeyBinding::new("ctrl-+", ZoomIn, Some(BOOK_VIEWER_CONTEXT)),
    KeyBinding::new("ctrl--", ZoomOut, Some(BOOK_VIEWER_CONTEXT)),
    KeyBinding::new("ctrl-0", ResetZoom, Some(BOOK_VIEWER_CONTEXT)),
    KeyBinding::new("ctrl-w", FitWidth, Some(BOOK_VIEWER_CONTEXT)),
    KeyBinding::new("ctrl-shift-p", FitPage, Some(BOOK_VIEWER_CONTEXT)),

    // View & Presentation
    KeyBinding::new("f11", ToggleFullscreen, Some(BOOK_VIEWER_CONTEXT)),
    KeyBinding::new("ctrl-shift-f", ToggleFullscreen, Some(BOOK_VIEWER_CONTEXT)),
    KeyBinding::new("ctrl-i", ToggleInvertColors, Some(BOOK_VIEWER_CONTEXT)),
    KeyBinding::new("ctrl-b", ToggleSidebar, Some(BOOK_VIEWER_CONTEXT)),

    // Sidebar tabs
    KeyBinding::new("alt-1", ToggleBookmarks, Some(BOOK_VIEWER_CONTEXT)),
    KeyBinding::new("alt-2", ToggleOutline, Some(BOOK_VIEWER_CONTEXT)),
    KeyBinding::new("alt-3", ToggleThumbnails, Some(BOOK_VIEWER_CONTEXT)),
    KeyBinding::new("alt-4", ToggleTts, Some(BOOK_VIEWER_CONTEXT)),

    // TTS audio
    KeyBinding::new("ctrl-space", TtsPlayPause, Some(BOOK_VIEWER_CONTEXT)),
    KeyBinding::new("ctrl-alt-right", TtsNext, Some(BOOK_VIEWER_CONTEXT)),
    KeyBinding::new("ctrl-alt-left", TtsPrev, Some(BOOK_VIEWER_CONTEXT)),
  ]
}
```

- [ ] **Step 2: Bind default keys and attach action handlers in `book_viewer/mod.rs`**

In `crates/libera-reader/src/ui/pages/book-viewer/mod.rs`:
- Register bindings on initialization: `cx.bind_keys(actions::default_keybindings());`
- Add `.key_context(actions::BOOK_VIEWER_CONTEXT)` on the viewer container `div()`.
- Add `.on_action(cx.listener(...))` for `NextPage`, `PrevPage`, `ZoomIn`, `ZoomOut`, `ResetZoom`, `FitWidth`, `FitPage`, `ToggleSearch`, `ToggleFullscreen`, `ToggleInvertColors`, `ToggleSidebar`.

- [ ] **Step 3: Unit test for default keybindings generation**

Add unit test in `actions.rs`:
```rust
#[test]
fn test_default_keybindings_non_empty() {
  let bindings = default_keybindings();
  assert!(!bindings.is_empty());
  assert!(bindings.iter().any(|b| b.action_type() == std::any::TypeId::of::<NextPage>()));
  assert!(bindings.iter().any(|b| b.action_type() == std::any::TypeId::of::<ToggleSearch>()));
}
```

---

### Task 2: Zoom Enhancements (`FitWidth`, `FitPage`, `Ctrl+Wheel`)

**Files:**
- Modify: `crates/libera-reader/src/ui/pages/book-viewer/state/zoom.rs`
- Modify: `crates/libera-reader/src/ui/pages/book-viewer/state/mod.rs`
- Modify: `crates/libera-reader/src/ui/pages/book-viewer/viewport/scroll_container.rs`
- Test: `crates/libera-reader/src/ui/pages/book-viewer/state/zoom.rs` (unit tests)

**Interfaces:**
- Consumes: `viewport_size: Size<Pixels>`, `page_size: PageDimensions`
- Produces: Dynamic calculation of `zoom_factor` for `FitWidth` and `FitPage`; `on_scroll_wheel` handler for Ctrl+Wheel zoom.

- [ ] **Step 1: Write unit test for `calculate_fit_width` and `calculate_fit_page`**

Add in `zoom.rs`:
```rust
#[test]
fn test_fit_zoom_calculations() {
  let page_dims = PageDimensions::new(595.0, 842.0);
  let available_width = 1200.0;
  let available_height = 900.0;

  let width_factor = calculate_fit_width(available_width, page_dims.width);
  assert!((width_factor - (1200.0 / 595.0)).abs() < 0.01);

  let page_factor = calculate_fit_page(available_height, page_dims.height);
  assert!((page_factor - (900.0 / 842.0)).abs() < 0.01);
}
```

- [ ] **Step 2: Implement fit calculations in `zoom.rs`**

```rust
pub fn calculate_fit_width(available_width: f32, page_width: f32) -> f32 {
  if page_width <= 0.0 { 1.0 } else { (available_width / page_width).clamp(0.25, 4.0) }
}

pub fn calculate_fit_page(available_height: f32, page_height: f32) -> f32 {
  if page_height <= 0.0 { 1.0 } else { (available_height / page_height).clamp(0.25, 4.0) }
}
```

- [ ] **Step 3: Update `BookViewerState::set_zoom` to accept viewport bounds**

When `ZoomPreset::FitWidth` or `FitPage` is active, calculate zoom factor using available width/height minus padding:
```rust
pub fn update_fit_zoom(&mut self, available_size: Size<Pixels>) {
  let page = self.page_size(self.current_page);
  match self.zoom_preset {
    ZoomPreset::FitWidth => {
      let w = f32::from(available_size.width) - 48.0; // padding
      self.zoom_factor = calculate_fit_width(w, page.width);
    }
    ZoomPreset::FitPage => {
      let h = f32::from(available_size.height) - 64.0; // gap + badge
      self.zoom_factor = calculate_fit_page(h, page.height);
    }
    _ => {}
  }
}
```

- [ ] **Step 4: Implement Ctrl + Wheel Zoom in `scroll_container.rs`**

Intercept mouse wheel scroll with Ctrl modifier:
```rust
.on_scroll_wheel(cx.listener(|this, event: &ScrollWheelEvent, window, cx| {
  if event.modifiers.control || event.modifiers.platform {
    let delta = event.delta.pixel_delta(px(20.0)).y;
    let step = if delta > px(0.0) { 0.10 } else { -0.10 };
    this.state.update(cx, |s, cx| {
      s.adjust_zoom_by(step);
      cx.notify();
    });
  }
}))
```

---

### Task 3: Keybindings Model & `native_db` Persistence

**Files:**
- Create: `crates/libera-reader/src/db/models/settings/keybindings.rs`
- Modify: `crates/libera-reader/src/db/models/settings/mod.rs`
- Test: `crates/libera-reader/src/db/models/settings/keybindings.rs` (unit tests)

**Interfaces:**
- Consumes: Action identifiers, key strings
- Produces: `KeybindingsConfig` with conflict validation, reset logic, and serialization

- [ ] **Step 1: Write unit test for `KeybindingsConfig` conflict detection**

```rust
#[test]
fn test_keybindings_conflict_detection() {
  let mut config = KeybindingsConfig::default();
  assert!(config.find_conflict("NextPage", "ctrl-f").is_some()); // ctrl-f is bound to ToggleSearch
  assert!(config.find_conflict("NextPage", "ctrl-shift-x").is_none());
}
```

- [ ] **Step 2: Implement `KeybindingActionInfo` and `KeybindingsConfig`**

Define action metadata (ID, category, localized title key, default key):
```rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum KeybindingCategory {
  Navigation,
  Search,
  Zoom,
  View,
  Sidebar,
  Audio,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct KeybindingItem {
  pub action_id: String,
  pub category: KeybindingCategory,
  pub custom_key: Option<String>,
  pub default_key: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct KeybindingsConfig {
  pub bindings: HashMap<String, String>, // action_id -> key string
}
```

- [ ] **Step 3: Add `keybindings` field to `Settings` model**

In `crates/libera-reader/src/db/models/settings/mod.rs`, add `pub keybindings: KeybindingsConfig` with `#[serde(default)]`.

---

### Task 4: Custom Keybindings Editor UI in Settings

**Files:**
- Create: `crates/libera-reader/src/ui/pages/settings/keybindings/mod.rs`
- Create: `crates/libera-reader/src/ui/pages/settings/keybindings/key_recorder.rs`
- Create: `crates/libera-reader/src/ui/pages/settings/keybindings/keybinding_row.rs`
- Modify: `crates/libera-reader/src/ui/pages/settings/mod.rs` (add Keybindings tab)
- Modify: `locales/en.yml`, `locales/ru.yml` (translations for actions and categories)

**Interfaces:**
- Consumes: `KeybindingsConfig`, `cx.theme()`, `cx.bind_keys()`
- Produces: Interactive settings page where users can rebind shortcuts, detect conflicts, and reset defaults

- [ ] **Step 1: Add localization strings to `en.yml` and `ru.yml`**

Add action labels:
- `components.settings.tabs.keybindings`
- `components.settings.keybindings.categories.*`
- `components.settings.keybindings.actions.*`
- `components.settings.keybindings.press_key`
- `components.settings.keybindings.conflict_warning`
- `components.settings.keybindings.reset_all`
- `components.settings.keybindings.reset_item`

- [ ] **Step 2: Implement `KeyRecorder` component**

Modal/popover component that listens to `on_key_down` on the window:
- Detects modifier keys (`Control`, `Alt`, `Shift`, `Super`) and main key.
- Converts to canonical GPUI key string (`"ctrl-shift-f"`, `"alt-1"`).
- Emits event with recorded key string.

- [ ] **Step 3: Implement `KeybindingRow` component**

Renders an action row:
- Action name and description.
- Button displaying current key combination (e.g. `Kbd` chip `"Ctrl + F"`).
- Clicking button opens `KeyRecorder`.
- Reset button (restores default key if custom key is set).

- [ ] **Step 4: Implement `KeybindingsView` in Settings**

Renders category sections with search/filter field and "Reset All" button.
On change: updates `cx.settings_mut().keybindings`, saves to `native_db`, and calls dynamic re-bind `actions::apply_custom_keybindings(cx)`.

---

### Task 5: Virtualize Thumbnails in Sidebar (`ThumbnailsView`) Reusing `BooksGrid` Architecture

**Files:**
- Create: `crates/libera-reader/src/ui/pages/book-viewer/sidebar/thumbnails/cache.rs`
- Create: `crates/libera-reader/src/ui/pages/book-viewer/sidebar/thumbnails/loader.rs`
- Modify: `crates/libera-reader/src/ui/pages/book-viewer/sidebar/thumbnails/mod.rs`
- Modify: `crates/libera-reader/src/ui/pages/book-viewer/sidebar/thumbnails/thumbnail_card.rs`

**Interfaces:**
- Consumes: `total_pages`, `book_path`, `mutool::render_page_to_png_bytes(..., 72)`, `v_virtual_list`
- Produces: High-performance virtualized page thumbnail grid matching the `BooksGrid` pattern (bounded LRU texture cache, GPU atlas flush on eviction, cancellable background loading, coalesced frame notifications)

- [ ] **Step 1: Port `BoundedCache` for page thumbnails (`cache.rs`)**

Implement LRU cache of decoded page thumbnail textures (`RenderImage`):
- Keyed by `page_number: usize`.
- Tracks `evicted: Vec<Arc<RenderImage>>`.
- `flush_evictions(&mut self, cx: &mut App)` calls `cx.drop_image(img, None)` to free GPU atlas memory.
- `pop_loading(&mut self, page: usize)` cancels pending off-screen loads.

- [ ] **Step 2: Port background worker loader with Semaphore (`loader.rs`)**

Spawn worker on Tokio runtime matching `BooksGrid::spawn_background_loader`:
- Bounded parallelism with `tokio::sync::Semaphore(threads)`.
- Checks `visible_start` and `visible_end` (cancels requests when user fast-scrolls past).
- Renders at `THUMBNAIL_DPI = 72` via `mutool::render_page_to_png_bytes`.
- Coalesced notify channel: drains pending updates to trigger only 1 UI refresh per frame.

- [ ] **Step 3: Virtualize `ThumbnailsView` using `v_virtual_list`**

In `sidebar/thumbnails/mod.rs`:
- Calculate 2-column row item sizes.
- Bind `VirtualListScrollHandle`.
- Render only visible 4..8 thumbnail cards in viewport.
- Highlight active page card with `cx.theme().primary` border.
- Clicking a card triggers `state.go_to_page(page)`.

- [ ] **Step 4: Verify thumbnail tab memory and rendering speed**

Open a 500+ page book, switch to Thumbnails tab, verify instantaneous appearance without stutter.

---

### Task 6: Verification & End-to-End Testing

**Files:**
- All touched files in `crates/libera-reader`

- [ ] **Step 1: Run unit tests**
Run: `cargo test -p libera-reader --lib -- actions::tests`
Run: `cargo test -p libera-reader --lib -- zoom::tests`
Run: `cargo test -p libera-reader --lib -- keybindings::tests`
Expected: ALL PASS

- [ ] **Step 2: Verify zero compiler warnings**
Run: `cargo check -p libera-reader --lib`
Expected: 0 warnings
