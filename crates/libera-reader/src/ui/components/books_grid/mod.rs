mod actions;
pub(crate) mod cache;
mod card;
pub(crate) mod image_utils;
pub(crate) mod loader;

use crate::TOKIO;
use crate::app_ext::AppExt;
use crate::books_state::{BooksState, TargetList};
use crate::db::models::CardDisplayMode;
use crate::db::models::books::book::Book;
use crate::ui::components::books_grid::cache::{BoundedCache, CoverState};
use crate::ui::components::books_grid::loader::spawn_background_loader;
use crate::ui::constants as C;
use gpui::{
  AsyncApp, Context, Div, IntoElement, ParentElement, Pixels, Render, Styled, Window, div, px, size,
};
use gpui_component::scroll::Scrollbar;
use gpui_component::{VirtualListScrollHandle, v_virtual_list};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

pub struct BooksGrid {
  pub(crate) state: gpui::Entity<BooksState>,
  pub(crate) target: TargetList,
  pub(crate) columns: usize,
  pub(crate) row_height: Pixels,
  pub(crate) mode: CardDisplayMode,
  pub(crate) scroll_handle: VirtualListScrollHandle,
  pub(crate) id: gpui::ElementId,
  pub(crate) thumbnail_paths: Vec<Option<PathBuf>>,
  pub(crate) image_cache: Arc<Mutex<BoundedCache>>,
  pub(crate) visible_start: Arc<AtomicUsize>,
  pub(crate) visible_end: Arc<AtomicUsize>,
  pub(crate) load_tx: tokio::sync::mpsc::UnboundedSender<(usize, gpui::SharedString, PathBuf)>,
}

impl BooksGrid {
  pub fn new(target: TargetList, id: gpui::ElementId, cx: &mut Context<Self>) -> Self {
    let state = cx.books_state_entity().clone();
    let cache_size = cx.settings().read().image_cache_size as usize;
    cx.observe(&state, |_, _, cx| cx.notify()).detach();
    let cache = Arc::new(Mutex::new(BoundedCache::new(cache_size.max(1))));
    let visible_start = Arc::new(AtomicUsize::new(0));
    let visible_end = Arc::new(AtomicUsize::new(0));

    let (load_tx, load_rx) =
      tokio::sync::mpsc::unbounded_channel::<(usize, gpui::SharedString, PathBuf)>();
    let (notify_tx, mut notify_rx) = tokio::sync::mpsc::unbounded_channel::<()>();

    cx.spawn(|this: gpui::WeakEntity<Self>, cx: &mut AsyncApp| {
      let mut owned_cx = cx.clone();
      async move {
        // Coalesce notify signals: drain all pending messages, then trigger a single
        // re-render. Without this, loading 50 covers in a burst would cause 50 separate
        // notify() calls and 50 frame rebuilds (see documentation/ram.md §10).
        while notify_rx.recv().await.is_some() {
          while notify_rx.try_recv().is_ok() {}
          let _ = this.update(&mut owned_cx, |_, cx| cx.notify());
        }
      }
    })
    .detach();

    let tokio_runtime = TOKIO.get().expect("Tokio runtime not initialized");
    spawn_background_loader(
      tokio_runtime,
      cache.clone(),
      visible_start.clone(),
      visible_end.clone(),
      load_rx,
      notify_tx,
    );

    Self {
      state,
      target,
      columns: 6,
      row_height: px(200.0),
      mode: CardDisplayMode::Compact,
      scroll_handle: VirtualListScrollHandle::new(),
      id,
      thumbnail_paths: Vec::new(),
      image_cache: cache,
      visible_start,
      visible_end,
      load_tx,
    }
  }

  /// Apply the dynamically computed layout (called from the page `render`).
  /// Mirrors the old `set_layout`: List mode halves the column count, Detailed
  /// mode reserves space for the footer.
  pub fn set_layout(&mut self, columns: usize, row_height: Pixels, mode: CardDisplayMode) {
    self.mode = mode;
    self.columns =
      if mode == CardDisplayMode::List { (columns / 2).max(1) } else { columns.max(1) };
    self.row_height = if mode == CardDisplayMode::Detailed {
      row_height + px(crate::ui::components::books_grid::FOOTER_HEIGHT_PX)
    } else {
      row_height
    };
  }

  /// Replace the resolved thumbnail paths (one entry per book in the current
  /// target list, in order; `None` means no thumbnail on disk yet).
  pub fn set_thumbnail_paths(&mut self, paths: Vec<Option<PathBuf>>, cx: &mut Context<Self>) {
    self.thumbnail_paths = paths;
    cx.notify();
  }

  fn total_books(&self, cx: &mut Context<Self>) -> usize {
    self.state.read(cx).total_books(self.target)
  }

  fn render_placeholder() -> Div {
    div().w_0().flex_grow_1().h_full().flex().flex_col().opacity(0.0)
  }
}

/// Footer height reserved in `Detailed` card mode.
pub(crate) const FOOTER_HEIGHT_PX: f32 = 52.0;

/// Per-cell data collected in pass 1 (with borrows held) and consumed in pass 2
/// (with a clean mutable view) to avoid borrow conflicts.
type RowCells = Vec<(Book, Option<PathBuf>, Option<CoverState>)>;

impl Render for BooksGrid {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    self.image_cache.lock().unwrap().flush_evictions(cx);

    let total_books = self.total_books(cx);
    let columns = self.columns.max(1);
    let total_rows = total_books.div_ceil(columns);
    let row_height = self.row_height;
    let card_height = row_height - px(C::GRID_ROW_GAP);
    let item_sizes: Rc<Vec<gpui::Size<Pixels>>> =
      Rc::new((0..total_rows.max(1)).map(|_| size(px(0.), row_height)).collect());

    let scroll = self.scroll_handle.clone();
    let list = v_virtual_list(
      cx.entity().clone(),
      self.id.clone(),
      item_sizes,
      move |view: &mut BooksGrid, visible_range, _window, cx| {
        let start_idx = visible_range.start * columns;
        let end_idx = (visible_range.end * columns).min(total_books);
        view.visible_start.store(start_idx, Ordering::Relaxed);
        view.visible_end.store(end_idx, Ordering::Relaxed);

        // --- Pass 1: gather per-row cell data while immutable borrows are held ---
        // `view.state` (read) and `view.image_cache` (mutex) are borrowed here.
        // Both must be released before pass 2 hands `view` to the card renderer.
        let mut rows_data: Vec<RowCells> = Vec::new();
        {
          let state_guard = view.state.read(cx).read();
          let keys = match view.target {
            TargetList::Library => &state_guard.library_keys,
            TargetList::Favorites => &state_guard.favorites_keys,
            TargetList::History => &state_guard.history_keys,
            TargetList::Bookmarks => &state_guard.bookmarks_keys,
          };
          let mut cache_lock = view.image_cache.lock().unwrap();

          for row in visible_range {
            if row >= total_rows {
              break;
            }
            let start = row * columns;
            let end = (start + columns).min(total_books);

            let mut cell_data: RowCells = Vec::with_capacity(end - start);
            for (i, id) in keys.iter().enumerate().take(end).skip(start) {
              let book = match state_guard.books_map.get(id) {
                Some(b) => b.clone(),
                None => continue,
              };
              let thumb = view.thumbnail_paths.get(i).cloned().flatten();

              // Cache is keyed by book id, so a reorder (reverse/sort/search)
              // never surfaces a stale cover from the old cell at this index.
              let cover_state = cache_lock.get_mut(id);
              if cover_state.is_none()
                && let Some(path) = thumb.clone()
              {
                cache_lock.insert(id.clone(), CoverState::Loading);
                let _ = view.load_tx.send((i, id.clone(), path));
              }
              cell_data.push((book, thumb, cover_state));
            }
            rows_data.push(cell_data);
          }
        }
        // borrows of `view.state` and `view.image_cache` are now released.

        // --- Pass 2: render cells with a clean mutable `view` ---
        let mut rows = Vec::with_capacity(rows_data.len());
        for cell_data in rows_data {
          let mut cells: Vec<Div> = Vec::with_capacity(columns);
          for (book, thumb, cover_state) in cell_data {
            cells.push(card::render_book_card(view, &book, thumb, cover_state, card_height, cx));
          }
          // Pad the row so columns stay aligned.
          for _ in cells.len()..columns {
            cells.push(Self::render_placeholder());
          }

          rows.push(
            div()
              .flex()
              .w_full()
              .h(row_height)
              .gap(px(C::GRID_CELL_GAP))
              .pl(px(C::GRID_PL))
              .pb(px(C::GRID_ROW_GAP))
              .children(cells),
          );
        }
        rows
      },
    )
    .track_scroll(&scroll)
    .size_full();

    div().relative().size_full().child(div().size_full().pr(px(C::GRID_PR)).child(list)).child(
      div()
        .absolute()
        .top_0()
        .right_0()
        .bottom_0()
        .w_2()
        .child(Scrollbar::new(&self.scroll_handle)),
    )
  }
}
