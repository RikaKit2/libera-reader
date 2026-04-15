use crate::books_state::{BooksState, TargetList};
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{ActiveTheme, VirtualListScrollHandle, scroll::Scrollbar, v_virtual_list};
use std::rc::Rc;

pub struct BookGridConfig<'a, V: Render> {
  pub view: &'a Entity<V>,
  pub id: ElementId,
  pub state: &'a Entity<BooksState>,
  pub target: TargetList,
  pub columns: usize,
  pub row_height: Pixels,
  pub scroll_handle: &'a VirtualListScrollHandle,
}

fn render_grid_item(is_book: bool, cx: &mut Context<impl Render>) -> Div {
  div()
    .w_0()
    .flex_grow()
    .h_full()
    .flex()
    .flex_col()
    .child(div().w_full().flex_1().mb_4().rounded_md().when(is_book, |this| {
      this.bg(cx.theme().border).border_1().border_color(cx.theme().border).flex().items_center().justify_center()
    }))
    .child(div().h_0().overflow_hidden())
}

pub fn book_virtual_grid<V: Render + 'static>(
  config: BookGridConfig<'_, V>, cx: &mut Context<V>,
) -> impl IntoElement {
  let total_books = {
    let books_state = config.state.read(cx);
    match config.target {
      TargetList::Library => books_state.library_keys.len(),
      TargetList::Favorites => books_state.favorites_keys.len(),
      TargetList::History => books_state.history_keys.len(),
    }
  };

  let columns = config.columns.max(1);
  let row_count = total_books.div_ceil(columns);
  let item_sizes = Rc::new((0..row_count).map(|_| size(px(0.), config.row_height)).collect::<Vec<_>>());

  let state_clone = config.state.clone();
  let target = config.target;
  let view = config.view.clone();
  let id = config.id;
  let scroll_handle = config.scroll_handle;

  div()
    .relative()
    .size_full()
    .child(
      div().size_full().pr_4().child(
        v_virtual_list(view, id, item_sizes, move |_this, visible_range, _window, cx| {
          let book_exists_flags: Vec<bool> = {
            let books_state = state_clone.read(cx);
            let all_keys = match target {
              TargetList::Library => &books_state.library_keys,
              TargetList::Favorites => &books_state.favorites_keys,
              TargetList::History => &books_state.history_keys,
            };

            all_keys.iter().map(|path| books_state.get_book(path).is_some()).collect()
          };

          visible_range
            .map(|row_index| {
              let start = row_index * columns;

              let mut row = div().flex().flex_row().w_full().h(config.row_height).gap_4().pl_2();

              for i in 0..columns {
                let book_exists = book_exists_flags.get(start + i).copied().unwrap_or(false);
                row = row.child(render_grid_item(book_exists, cx));
              }

              row
            })
            .collect::<Vec<_>>()
        })
        .track_scroll(scroll_handle)
        .size_full(),
      ),
    )
    .child(div().absolute().top_0().right_0().bottom_0().w_2().child(Scrollbar::new(scroll_handle)))
}
