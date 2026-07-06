use crate::books_state::BooksState;
use crate::ui::components::BooksGrid;
use crate::ui::constants as C;
use gpui::{
  AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Subscription, Window,
  div, px, size,
};
use gpui_component::ActiveTheme;
use libera_reader_core::ctx::Ctx;
use libera_reader_core::db::models::books::book::BookPath;
use std::rc::Rc;

pub(crate) struct Favorite {
  book_grid: Entity<BooksGrid>,
  _subscriptions: Vec<Subscription>,
}

fn resolve_thumbnails(
  keys: &[gpui::SharedString], db: &libera_reader_core::db::DB, thumbnails_dir: &std::path::Path,
) -> Vec<std::path::PathBuf> {
  keys
    .iter()
    .filter_map(|id| {
      let book_path = BookPath::from_id(id);
      let book = db.get_book(book_path).ok()??;
      let png_path = book.get_thumbnail_png_path(db, thumbnails_dir).ok()??;
      if png_path.exists() { Some(png_path) } else { None }
    })
    .collect()
}

impl Favorite {
  pub fn new(
    _window: &mut Window, cx: &mut Context<Self>, books_state: Entity<BooksState>,
  ) -> Self {
    let thumbnails_dir = Ctx::global(cx).app_dirs.read().thumbnails_dir.clone();
    let db = Ctx::global(cx).db.clone();
    let cache_size = Ctx::global(cx).settings.read().image_cache_size as usize;

    let images = resolve_thumbnails(&books_state.read(cx).favorites_keys, &db, &thumbnails_dir);
    let book_grid = cx.new(|cx| BooksGrid::new(images, cache_size, cx));

    let _subscriptions = vec![cx.observe(&books_state, |this, _state, cx| {
      let thumbnails_dir = Ctx::global(cx).app_dirs.read().thumbnails_dir.clone();
      let db = Ctx::global(cx).db.clone();
      let images = resolve_thumbnails(&_state.read(cx).favorites_keys, &db, &thumbnails_dir);
      let rows = images.len().div_ceil(6);
      let sizes = Rc::new(std::iter::repeat_n(size(px(800.0), px(160.0)), rows.max(1)).collect());
      this.book_grid.update(cx, |grid, _cx| {
        grid.images = images;
        grid.item_sizes = sizes;
      });
      cx.notify();
    })];

    Self { book_grid, _subscriptions }
  }
}

impl Render for Favorite {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    div().w_full().h_full().flex().flex_col().text_color(cx.theme().foreground).children([div()
      .bg(cx.theme().background)
      .w_full()
      .h_full()
      .pt(px(C::GRID_PT))
      .child(self.book_grid.clone())])
  }
}
