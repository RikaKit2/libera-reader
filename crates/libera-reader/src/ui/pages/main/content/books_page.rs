use crate::app_ext::AppExt;
use crate::books_state::TargetList;
use crate::ui::components::{BooksGrid, TopBar};
use crate::ui::pages::main::constants as C;
use gpui::{
  AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Subscription, Window,
  div, px,
};
use gpui_component::ActiveTheme;

pub(crate) struct BooksPage {
  top_bar: Entity<TopBar>,
  book_grid: Entity<BooksGrid>,
  _subscriptions: Vec<Subscription>,
}

impl BooksPage {
  pub fn new(window: &mut Window, cx: &mut Context<Self>, target: TargetList) -> Self {
    let books_state = cx.books_state_entity().clone();
    let top_bar = cx.new(|cx| TopBar::new(window, cx, target));

    let grid_id = match target {
      TargetList::Library => "library-grid",
      TargetList::Favorites => "favorites-grid",
      TargetList::History => "history-grid",
      TargetList::Bookmarks => "bookmarks-grid",
    };
    let book_grid = cx.new(|cx| BooksGrid::new(target, grid_id.into(), cx));

    let _subscriptions = vec![cx.observe(&books_state, move |_this, _state, cx| {
      cx.notify();
    })];

    Self { top_bar, book_grid, _subscriptions }
  }
}
impl Render for BooksPage {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
