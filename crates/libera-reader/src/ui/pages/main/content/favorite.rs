use crate::app_ext::AppExt;
use crate::books_state::TargetList;
use crate::ui::components::{BooksGrid, TopBar};
use crate::ui::constants as C;
use gpui::{
  AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Subscription, Window,
  div, px,
};
use gpui_component::ActiveTheme;

pub(crate) struct Favorite {
  top_bar: Entity<TopBar>,
  book_grid: Entity<BooksGrid>,
  _subscriptions: Vec<Subscription>,
}

impl Favorite {
  pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
    let books_state = cx.books_state_entity().clone();
    let top_bar = cx.new(|cx| TopBar::new(window, cx, TargetList::Favorites));

    let db = cx.db().clone();
    let keys = books_state.read(cx).favorites_keys();
    let paths = books_state.read(cx).collect_thumbnail_paths(&keys, &db);
    let book_grid = cx.new(|cx| BooksGrid::new(TargetList::Favorites, "favorites-grid".into(), cx));
    book_grid.update(cx, |grid, cx| grid.set_thumbnail_paths(paths, cx));

    let _subscriptions = vec![cx.observe(&books_state, move |this, state, cx| {
      let db = cx.db().clone();
      let keys = state.read(cx).favorites_keys();
      let paths = state.read(cx).collect_thumbnail_paths(&keys, &db);
      this.book_grid.update(cx, |grid, cx| grid.set_thumbnail_paths(paths, cx));
      cx.notify();
    })];

    Self { top_bar, book_grid, _subscriptions }
  }
}

impl Render for Favorite {
  fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let (columns, ui_zoom) = {
      let s = cx.settings().read();
      (s.number_of_columns as usize, s.ui_zoom)
    };
    let available_width =
      window.viewport_size().width - px(C::SIDEBAR_W + C::GRID_PL + C::GRID_PR + C::SCROLLBAR_W);
    let col_width = available_width / columns as f32;
    let row_height = col_width * C::COVER_RATIO * ui_zoom as f32 + px(C::GRID_ROW_HEIGHT_EXTRA);
    let mode = cx.settings().read().card_display_mode;
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
