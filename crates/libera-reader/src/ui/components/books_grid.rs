use crate::books_state::{BooksState, TargetList};
use crate::ui::constants as C;
use gpui::*;
use gpui_component::{ActiveTheme, VirtualListScrollHandle, scroll::Scrollbar, v_virtual_list};
use libera_reader_core::db::models::books::book::{Book, BookPath};
use std::rc::Rc;

pub struct BooksGrid {
  state: Entity<BooksState>,
  target: TargetList,
  columns: usize,
  row_height: Pixels,
  scroll_handle: VirtualListScrollHandle,
  id: ElementId,
}

impl BooksGrid {
  pub fn new(
    state: Entity<BooksState>, target: TargetList, columns: usize, row_height: Pixels, id: ElementId,
  ) -> Self {
    Self { state, target, columns: columns.max(1), row_height, scroll_handle: VirtualListScrollHandle::new(), id }
  }

  pub fn set_layout(&mut self, columns: usize, row_height: Pixels) {
    self.columns = columns.max(1);
    self.row_height = row_height;
  }

  fn keys_for_target(state: &BooksState, target: TargetList) -> &Vec<BookPath> {
    match target {
      TargetList::Library => &state.library_keys,
      TargetList::Favorites => &state.favorites_keys,
      TargetList::History => &state.history_keys,
      TargetList::Bookmarks => &state.bookmarks_keys,
    }
  }

  fn total_books(&self, cx: &mut Context<Self>) -> usize {
    let state = self.state.read(cx);
    Self::keys_for_target(state, self.target).len()
  }

  fn total_rows(&self, total_books: usize) -> usize {
    total_books.div_ceil(self.columns)
  }

  fn item_sizes(&self, total_books: usize) -> Rc<Vec<Size<Pixels>>> {
    let row_count = self.total_rows(total_books);
    Rc::new((0..row_count).map(|_| size(px(0.), self.row_height)).collect::<Vec<_>>())
  }

  fn render_book_cover(border_color: Hsla) -> Div {
    div()
      .w_full()
      .flex_1()
      .rounded_md()
      .bg(border_color)
      .border_1()
      .border_color(border_color)
      .flex()
      .items_center()
      .justify_center()
  }

  fn render_book_title(book_name: SharedString) -> Div {
    div()
      .w_full()
      .h(px(36.))
      .pt_1()
      .overflow_hidden()
      .child(div().w_full().whitespace_normal().line_clamp(2).text_xs().line_height(rems(1.1)).child(book_name))
  }

  fn render_book_card(book: &Book, border_color: Hsla) -> Div {
    div()
      .w_0()
      .flex_grow()
      .h_full()
      .flex()
      .flex_col()
      .overflow_hidden()
      .child(Self::render_book_cover(border_color))
      .child(Self::render_book_title(book.book_path.display_name()))
  }

  fn render_placeholder() -> Div {
    div()
      .w_0()
      .flex_grow()
      .h_full()
      .flex()
      .flex_col()
      .opacity(0.0)
      .child(div().w_full().flex_1().rounded_md())
      .child(div().h_0().overflow_hidden())
  }

  fn render_row(&self, row_index: usize, total_books: usize, cx: &mut Context<Self>) -> Div {
    let mut row = div()
      .flex()
      .flex_row()
      .w_full()
      .h(self.row_height)
      .gap(px(C::GRID_CELL_GAP))
      .pl(px(C::GRID_PL))
      .pb(px(C::GRID_ROW_GAP));

    let start = row_index * self.columns;
    if start >= total_books {
      return row;
    }

    let border_color = cx.theme().border;
    let state = self.state.read(cx);
    let keys = Self::keys_for_target(state, self.target);
    let end = (start + self.columns).min(total_books);
    let row_keys = &keys[start..end];

    let mut rendered_books = 0usize;
    for path in row_keys {
      if let Some(book) = state.get_book(path) {
        row = row.child(Self::render_book_card(book, border_color));
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
          v_virtual_list(cx.entity(), self.id.clone(), item_sizes, move |view, visible_range, _window, cx| {
            visible_range
              .filter(|&row_index| row_index < total_rows)
              .map(|row_index| view.render_row(row_index, total_books, cx))
              .collect::<Vec<_>>()
          })
          .track_scroll(&self.scroll_handle)
          .size_full(),
        ),
      )
      .child(div().absolute().top_0().right_0().bottom_0().w_2().child(Scrollbar::new(&self.scroll_handle)))
  }
}
