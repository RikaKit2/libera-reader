use crate::books_state::{BooksState, TargetList};
use crate::ui::components::{BooksGrid, TopBar};
use crate::ui::constants as C;
use gpui::{
  AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Subscription, Window,
  div, px,
};
use gpui_component::ActiveTheme;
use libera_reader_core::ctx::Ctx;
use libera_reader_core::db::models::books::book::BookPath;
use std::path::{Path, PathBuf};

pub(crate) struct Favorite {
  top_bar: Entity<TopBar>,
  book_grid: Entity<BooksGrid>,
  _subscriptions: Vec<Subscription>,
}

fn resolve_thumbnail_paths(
  keys: &[gpui::SharedString], db: &libera_reader_core::db::DB, thumbnails_dir: &Path,
) -> Vec<Option<PathBuf>> {
  keys
    .iter()
    .map(|id| {
      let book_path = BookPath::from_id(id);
      let book = db.get_book(book_path).ok()??;
      let png_path = book.get_thumbnail_png_path(db, thumbnails_dir).ok()??;
      if png_path.exists() { Some(png_path) } else { None }
    })
    .collect()
}

impl Favorite {
  pub fn new(window: &mut Window, cx: &mut Context<Self>, books_state: Entity<BooksState>) -> Self {
    let top_bar = cx.new(|cx| TopBar::new(window, cx, books_state.clone(), TargetList::Favorites));

    let thumbnails_dir = Ctx::global(cx).app_dirs.read().thumbnails_dir.clone();
    let db = Ctx::global(cx).db.clone();
    let cache_size = Ctx::global(cx).settings.read().image_cache_size as usize;
    let paths = resolve_thumbnail_paths(&books_state.read(cx).favorites_keys, &db, &thumbnails_dir);
    let book_grid = cx.new(|cx| {
      BooksGrid::new(
        books_state.clone(),
        TargetList::Favorites,
        cache_size,
        "favorites-grid".into(),
        cx,
      )
    });
    book_grid.update(cx, |grid, cx| grid.set_thumbnail_paths(paths, cx));

    let _subscriptions = vec![cx.observe(&books_state, move |this, state, cx| {
      let thumbnails_dir = Ctx::global(cx).app_dirs.read().thumbnails_dir.clone();
      let db = Ctx::global(cx).db.clone();
      let paths = resolve_thumbnail_paths(&state.read(cx).favorites_keys, &db, &thumbnails_dir);
      this.book_grid.update(cx, |grid, cx| grid.set_thumbnail_paths(paths, cx));
      cx.notify();
    })];

    Self { top_bar, book_grid, _subscriptions }
  }
}

impl Render for Favorite {
  fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let (columns, ui_zoom) = {
      let s = Ctx::global(cx).settings.read();
      (s.number_of_columns as usize, s.ui_zoom)
    };
    let available_width =
      window.viewport_size().width - px(C::SIDEBAR_W + C::GRID_PL + C::GRID_PR + C::SCROLLBAR_W);
    let col_width = available_width / columns as f32;
    let row_height = col_width * C::COVER_RATIO * ui_zoom as f32 + px(C::GRID_ROW_HEIGHT_EXTRA);
    let mode = Ctx::global(cx).settings.read().card_display_mode;
    self.book_grid.update(cx, |grid, _cx| grid.set_layout(columns, row_height, mode));

    div()
      .w_full()
      .h_full()
      .flex()
      .flex_col()
      .text_color(cx.theme().foreground)
      .child(self.top_bar.clone())
      .child(
        div()
          .bg(cx.theme().background)
          .w_full()
          .h_full()
          .pt(px(C::GRID_PT))
          .child(self.book_grid.clone()),
      )
  }
}
