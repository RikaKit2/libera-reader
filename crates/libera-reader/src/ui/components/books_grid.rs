use crate::books_state::{BooksState, TargetList};
use crate::ui::constants as C;
use gpui::*;
use gpui_component::{
  ActiveTheme, Icon, IconName, Sizable, VirtualListScrollHandle, button::Button,
  button::ButtonCustomVariant, button::ButtonVariants, scroll::Scrollbar, v_virtual_list,
};
use libera_reader_core::ctx::Ctx;
use libera_reader_core::db::models::{
  CardDisplayMode,
  books::book::{Book, BookPath},
};
use std::{cell::RefCell, collections::HashMap, path::PathBuf, rc::Rc};

fn push_at_history(path: BookPath, books_state: Entity<BooksState>, cx: &mut App) {
  let mut should_persist = false;
  books_state.update(cx, |state, cx| {
    if let Some(dir_books) = state.books_map.get_mut(&path.parent_dir) {
      let key = path.file_name();
      if let Some(book) = dir_books.storage.get_mut(&key)
        && !book.user_data.in_history
      {
        book.user_data.in_history = true;
        should_persist = true;
        if !state.history_keys.contains(&path) {
          state.history_keys.push(path.clone());
          state.apply_sorting_to(TargetList::History);
        }
        cx.notify();
      }
    }
  });
  if should_persist {
    cx.update_global(|ctx: &mut Ctx, _cx| {
      if let Ok(Some(book)) = ctx.db.get_book(path.clone()) {
        let mut updated_book = book.clone();
        updated_book.user_data.in_history = true;
        let _ = ctx.db.update_book(updated_book);
      }
    });
  }
}

fn toggle_favorite(path: BookPath, books_state: Entity<BooksState>, cx: &mut App) {
  let mut next_favorite_state = None;
  books_state.update(cx, |state, cx| {
    if let Some(dir_books) = state.books_map.get_mut(&path.parent_dir) {
      let key = path.file_name();
      if let Some(book) = dir_books.storage.get_mut(&key) {
        book.user_data.favorite = !book.user_data.favorite;
        next_favorite_state = Some(book.user_data.favorite);
        if book.user_data.favorite {
          if !state.favorites_keys.contains(&path) {
            state.favorites_keys.push(path.clone());
            state.apply_sorting_to(TargetList::Favorites);
          }
        } else {
          state.favorites_keys.retain(|k| k != &path);
        }
        cx.notify();
      }
    }
  });
  if let Some(is_favorite) = next_favorite_state {
    cx.update_global(|ctx: &mut Ctx, _cx| {
      if let Ok(Some(book)) = ctx.db.get_book(path.clone()) {
        let mut updated_book = book.clone();
        updated_book.user_data.favorite = is_favorite;
        let _ = ctx.db.update_book(updated_book);
      }
    });
  }
}

pub struct BooksGrid {
  state: Entity<BooksState>,
  target: TargetList,
  columns: usize,
  row_height: Pixels,
  scroll_handle: VirtualListScrollHandle,
  id: ElementId,
  title_cache: RefCell<HashMap<BookPath, SharedString>>,
}

impl BooksGrid {
  const TITLE_BREAKABLE_CAPACITY_MULTIPLIER: usize = 4;
  const TITLE_FONT_SIZE_REM: f32 = 0.9;
  const TITLE_LINE_HEIGHT_REM: f32 = 1.1;
  const FAVORITE_INACTIVE_OPACITY: f32 = 0.7;
  const FOOTER_HEIGHT_PX: f32 = 52.0;
  const COVER_ID_PREFIX: &'static str = "cover_";
  const FAVORITE_ID_PREFIX: &'static str = "fav_";

  pub fn new(
    state: Entity<BooksState>, target: TargetList, columns: usize, row_height: Pixels,
    id: ElementId, cx: &mut Context<Self>,
  ) -> Self {
    // IMPORTANT: subscribe to BooksState events so GPUI will re-render the grid
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
    self.columns = columns.max(1);
    self.row_height = if mode == CardDisplayMode::Detailed {
      row_height + px(Self::FOOTER_HEIGHT_PX)
    } else {
      row_height
    };
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

  fn format_title_pixel_perfect(title: &str) -> SharedString {
    let mut breakable_title =
      String::with_capacity(title.len() * Self::TITLE_BREAKABLE_CAPACITY_MULTIPLIER);
    for ch in title.chars() {
      breakable_title.push(ch);
      breakable_title.push('\u{200B}');
    }
    breakable_title.into()
  }

  fn get_cached_title(&self, book_path: &BookPath) -> SharedString {
    let mut cache = self.title_cache.borrow_mut();
    if let Some(title) = cache.get(book_path) {
      return title.clone();
    }
    let display_name = book_path.display_name();
    let new_title = Self::format_title_pixel_perfect(display_name.as_ref());
    cache.insert(book_path.clone(), new_title.clone());
    new_title
  }

  fn render_book_title(&self, book_path: &BookPath, foreground: Hsla) -> Div {
    let display_name = self.get_cached_title(book_path);
    div().flex_1().min_w_0().pt_1().child(
      div()
        .w_full()
        .whitespace_normal()
        .line_clamp(2)
        .text_ellipsis()
        .overflow_hidden()
        .text_size(rems(Self::TITLE_FONT_SIZE_REM))
        .line_height(rems(Self::TITLE_LINE_HEIGHT_REM))
        .text_color(foreground)
        .child(display_name),
    )
  }

  fn render_cover_click_area(
    &self, book_path: BookPath, id: SharedString, has_thumbnail: bool, thumbnail_path: PathBuf,
    cx: &Context<Self>,
  ) -> Div {
    let theme = cx.theme();
    let state_hist = self.state.clone();
    let mut cover = div()
      .w_full()
      .h_full()
      .bg(theme.border)
      .relative()
      .flex()
      .items_center()
      .justify_center()
      .cursor_pointer();

    if has_thumbnail {
      cover = cover.child(img(thumbnail_path).w_full().h_full().object_fit(ObjectFit::Cover));
    } else {
      // Icon placeholder
      cover = cover.child(
        Icon::new(IconName::BookOpen).with_size(px(40.0)).text_color(theme.foreground.opacity(0.3)),
      );
    }

    let subtle_hover = ButtonCustomVariant::new(cx)
      .hover(gpui::white().opacity(0.12))
      .active(gpui::white().opacity(0.16));
    cover.child(
      Button::new(id)
        .custom(subtle_hover)
        .xsmall()
        .absolute()
        .inset_0()
        .w_full()
        .h_full()
        .on_click(move |_ev, _window, cx| {
          push_at_history(book_path.clone(), state_hist.clone(), cx);
        }),
    )
  }

  fn render_favorite_button(
    &self, book: &Book, id: SharedString, foreground: Hsla, primary: Hsla,
  ) -> Div {
    let favorite_path = book.book_path.clone();
    let state_fav = self.state.clone();
    let is_favorite = book.user_data.favorite;
    let fav_icon_source =
      if is_favorite { "heroicons--star-solid.svg" } else { "heroicons--star.svg" };
    div()
      .flex_shrink_0()
      .h_full()
      .flex()
      .items_center()
      .justify_center()
      .text_color(if is_favorite {
        primary
      } else {
        foreground.opacity(Self::FAVORITE_INACTIVE_OPACITY)
      })
      .hover(move |h| h.text_color(primary))
      .child(
        Button::new(id)
          .ghost()
          .xsmall()
          .on_click(move |_ev, _window, cx| {
            toggle_favorite(favorite_path.clone(), state_fav.clone(), cx);
          })
          .text()
          .icon(Icon::new(Icon::empty()).path(fav_icon_source).text_color(primary))
          .with_size(gpui_component::Size::Large),
      )
  }

  fn render_book_footer(&self, book: &Book, foreground: Hsla, primary: Hsla) -> Div {
    let title = self.render_book_title(&book.book_path, foreground);
    let fav_id: SharedString =
      format!("{}{}", Self::FAVORITE_ID_PREFIX, book.book_path.full_path_string()).into();
    div()
      .w_full()
      .h(px(Self::FOOTER_HEIGHT_PX))
      .flex_shrink_0()
      .flex()
      .flex_row()
      .items_center()
      .justify_between()
      .gap_1()
      .px_2()
      .py_1()
      .child(title)
      .child(self.render_favorite_button(book, fav_id, foreground, primary))
  }

  fn render_book_card(
    &self, book: &Book, has_thumbnail: bool, thumbnail_path: PathBuf, card_height: Pixels,
    cx: &Context<Self>,
  ) -> Div {
    let theme = cx.theme();
    let mode = Ctx::global(cx).settings.read().card_display_mode;
    let cover_id: SharedString =
      format!("{}{}", Self::COVER_ID_PREFIX, book.book_path.full_path_string()).into();

    let footer_height =
      if mode == CardDisplayMode::Detailed { px(Self::FOOTER_HEIGHT_PX) } else { px(0.0) };
    let cover_height = card_height - footer_height;

    let mut card = div()
      .w_0()
      .flex_grow()
      .h(card_height)
      .flex_shrink_0()
      .flex()
      .flex_col()
      .rounded_md()
      .bg(theme.background)
      .border_1()
      .border_color(theme.border)
      .overflow_hidden()
      .child(div().w_full().h(cover_height).flex_shrink_0().child(self.render_cover_click_area(
        book.book_path.clone(),
        cover_id,
        has_thumbnail,
        thumbnail_path,
        cx,
      )));

    if mode == CardDisplayMode::Detailed {
      card = card.child(self.render_book_footer(book, theme.foreground, theme.primary));
    }
    card
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
    let keys = Self::keys_for_target(state, self.target);
    let end = (start + self.columns).min(total_books);
    let row_keys = &keys[start..end];
    let mut rendered_books = 0usize;

    for path in row_keys {
      if let Some(book) = state.get_book(path) {
        let thumbnail_path = thumbnails_dir.join(path.file_name().as_ref()).with_extension("png");
        let has_thumbnail = thumbnail_path.exists();

        if !has_thumbnail {
          let ctx = Ctx::global(cx);
          let mut processing = ctx.not_cached_books.inner().processing_now.write().unwrap();

          // Check to prevent queue spam from causing hangs
          if !processing.contains(path) {
            processing.insert(path.clone());
            let _ = ctx.not_cached_books.high_tx().send(path.clone());
          }
        }

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
