mod actions;
mod card;

use crate::books_state::{BooksState, TargetList};
use crate::ui::constants as C;
use gpui::*;
use gpui_component::{VirtualListScrollHandle, scroll::Scrollbar, v_virtual_list};
use libera_reader_core::ctx::Ctx;
use libera_reader_core::db::models::CardDisplayMode;
use libera_reader_core::db::models::books::book::BookPath;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub(crate) const TITLE_BREAKABLE_CAPACITY_MULTIPLIER: usize = 4;
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
  pub(crate) title_cache: RefCell<HashMap<BookPath, SharedString>>,
}

impl BooksGrid {
  pub fn new(
    state: Entity<BooksState>, target: TargetList, columns: usize, row_height: Pixels,
    id: ElementId, cx: &mut Context<Self>,
  ) -> Self {
    cx.observe(&state, |_, _, cx| cx.notify()).detach();
    Self {
      state,
      target,
      columns: columns.max(1),
      row_height,
      scroll_handle: VirtualListScrollHandle::new(),
      id,
      title_cache: RefCell::new(HashMap::new()),
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

    let thumbnails_dir = Ctx::global(cx).app_dirs.read().thumbnails_dir.join("unhashed_books");
    let state = self.state.read(cx);
    let cover_cache = state.cover_cache.clone();

    let keys = match self.target {
      TargetList::Library => &state.library_keys,
      TargetList::Favorites => &state.favorites_keys,
      TargetList::History => &state.history_keys,
      TargetList::Bookmarks => &state.bookmarks_keys,
    };

    let end = (start + self.columns).min(total_books);
    let row_keys = &keys[start..end];
    let mut rendered_books = 0usize;

    for path in row_keys {
      if let Some(book) = state.get_book(path) {
        let thumbnail_path = thumbnails_dir.join(path.file_name().as_ref()).with_extension("png");

        // --- SIMPLE COVER CHECK: no disk I/O, just read from in-memory cache ---
        // The data_extraction_service processes books sequentially and emits
        // ThumbnailExtracted when a thumbnail is ready, which sets this to true.
        let has_thumbnail = {
          let cache = cover_cache.borrow();
          *cache.get(path).unwrap_or(&false)
        };

        row =
          row.child(self.render_book_card(book, has_thumbnail, thumbnail_path, card_height, cx));
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
