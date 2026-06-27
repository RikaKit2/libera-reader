mod actions;
mod card;
mod lru_cache;

use crate::books_state::{BooksState, TargetList};
use crate::ui::components::books_grid::lru_cache::LruImageCache;
use crate::ui::constants as C;
use gpui::*;
use gpui_component::{VirtualListScrollHandle, scroll::Scrollbar, v_virtual_list};
use libera_reader_core::ctx::Ctx;
use libera_reader_core::db::models::CardDisplayMode;
use std::cell::RefCell;
use std::rc::Rc;

pub(crate) const TITLE_FONT_SIZE_REM: f32 = 0.9;
pub(crate) const TITLE_LINE_HEIGHT_REM: f32 = 1.1;
pub(crate) const FAVORITE_INACTIVE_OPACITY: f32 = 0.7;
pub(crate) const FOOTER_HEIGHT_PX: f32 = 52.0;
pub(crate) const COVER_ID_PREFIX: &str = "cover_";
pub(crate) const FAVORITE_ID_PREFIX: &str = "fav_";

pub struct BooksGrid {
  pub(crate) state: Entity<BooksState>,
  pub(crate) target: TargetList,
  pub(crate) columns: usize,
  pub(crate) row_height: Pixels,
  pub(crate) scroll_handle: VirtualListScrollHandle,
  pub(crate) id: ElementId,
  pub(crate) image_cache: RefCell<LruImageCache>,
}

impl BooksGrid {
  pub fn new(
    state: Entity<BooksState>, target: TargetList, columns: usize, row_height: Pixels,
    id: ElementId, cx: &mut Context<Self>,
  ) -> Self {
    cx.observe(&state, |_, _, cx| cx.notify()).detach();

    // LRU cache with ~80 images (reduced from 200 for memory efficiency)
    let image_cache = RefCell::new(LruImageCache::new(80));

    let mut event_rx = Ctx::global(cx).event_tx.subscribe();
    cx.spawn(|this: gpui::WeakEntity<BooksGrid>, cx: &mut gpui::AsyncApp| {
      let mut owned_cx = cx.clone();
      async move {
        loop {
          match event_rx.recv().await {
            Ok(event) => {
              if let libera_reader_core::types::LibraryEvent::ThumbnailExtracted(path) = event {
                let id = path.full_path_string();
                let _ = this.update(&mut owned_cx, |grid, cx| {
                  if let Ok(mut cache) = grid.image_cache.try_borrow_mut() {
                    cache.remove(&id);
                  }
                  cx.notify();
                });
              }
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
              continue;
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
              break;
            }
          }
        }
      }
    })
    .detach();

    Self {
      state,
      target,
      columns: columns.max(1),
      row_height,
      scroll_handle: VirtualListScrollHandle::new(),
      id,
      image_cache,
    }
  }

  pub fn set_layout(&mut self, columns: usize, row_height: Pixels, mode: CardDisplayMode) {
    self.columns =
      if mode == CardDisplayMode::List { (columns / 2).max(1) } else { columns.max(1) };

    self.row_height = if mode == CardDisplayMode::Detailed {
      row_height + px(FOOTER_HEIGHT_PX)
    } else {
      row_height
    };
  }

  fn total_books(&self, cx: &mut Context<Self>) -> usize {
    let state = self.state.read(cx);
    match self.target {
      TargetList::Library => state.library_keys.len(),
      TargetList::Favorites => state.favorites_keys.len(),
      TargetList::History => state.history_keys.len(),
      TargetList::Bookmarks => state.bookmarks_keys.len(),
    }
  }

  fn total_rows(&self, total_books: usize) -> usize {
    total_books.div_ceil(self.columns)
  }

  fn item_sizes(&self, total_books: usize) -> Rc<Vec<Size<Pixels>>> {
    let row_count = self.total_rows(total_books);
    Rc::new((0..row_count).map(|_| size(px(0.), self.row_height)).collect::<Vec<_>>())
  }

  fn render_placeholder() -> Div {
    div().w_0().flex_grow().h_full().flex().flex_col().opacity(0.0)
  }

  fn render_row(&self, row_index: usize, total_books: usize, cx: &mut Context<Self>) -> Div {
    // Evict old images from GPUI asset cache to keep RAM extremely low
    if let Ok(mut cache) = self.image_cache.try_borrow_mut() {
      cache.flush_evictions(cx);
    }

    let card_height = self.row_height - px(C::GRID_ROW_GAP);
    let mut row = div()
      .flex()
      .flex_row()
      .w_full()
      .h(self.row_height)
      .gap(px(C::GRID_CELL_GAP))
      .pl(px(C::GRID_PL))
      .items_start()
      .pb(px(C::GRID_ROW_GAP));

    let start = row_index * self.columns;
    if start >= total_books {
      return row;
    }

    let state = self.state.read(cx);

    let keys = match self.target {
      TargetList::Library => &state.library_keys,
      TargetList::Favorites => &state.favorites_keys,
      TargetList::History => &state.history_keys,
      TargetList::Bookmarks => &state.bookmarks_keys,
    };

    let end = (start + self.columns).min(total_books);
    let row_keys = &keys[start..end];
    let mut rendered_books = 0usize;

    for id in row_keys {
      if let Some(light_book) = state.get_light_book(id) {
        row = row.child(self.render_book_card(light_book, card_height, cx));
        rendered_books += 1;
      }
    }

    let placeholders = self.columns.saturating_sub(rendered_books);
    for _ in 0..placeholders {
      row = row.child(Self::render_placeholder());
    }
    row
  }
}

impl Render for BooksGrid {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let total_books = self.total_books(cx);
    let total_rows = self.total_rows(total_books);
    let item_sizes = self.item_sizes(total_books);

    div()
      .relative()
      .size_full()
      .child(
        div().size_full().pr(px(C::GRID_PR)).child(
          v_virtual_list(
            cx.entity(),
            self.id.clone(),
            item_sizes,
            move |view, visible_range, _window, cx| {
              visible_range
                .filter(|&row_index| row_index < total_rows)
                .map(|row_index| view.render_row(row_index, total_books, cx))
                .collect::<Vec<_>>()
            },
          )
          .track_scroll(&self.scroll_handle)
          .size_full(),
        ),
      )
      .child(
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
