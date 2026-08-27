pub mod add_bookmark_btn;
pub mod bookmark_item;
pub mod bookmarks_list;
pub mod quick_bookmark_btn;

pub use add_bookmark_btn::AddBookmarkBtn;
pub use bookmarks_list::BookmarksList;
pub use quick_bookmark_btn::QuickBookmarkBtn;

use crate::ui::pages::book_viewer::state::BookViewerState;
use gpui::*;
use gpui_component::scroll::ScrollableElement;

pub struct BookmarksView {
  state: Entity<BookViewerState>,
  add_btn: Entity<AddBookmarkBtn>,
  quick_btn: Entity<QuickBookmarkBtn>,
  list: Entity<BookmarksList>,
}

impl BookmarksView {
  pub fn new(state: Entity<BookViewerState>, cx: &mut App) -> Entity<Self> {
    let add_btn = cx.new(|_cx| AddBookmarkBtn::new(state.clone()));
    let quick_btn = cx.new(|_cx| QuickBookmarkBtn::new(state.clone()));
    let list = cx.new(|_cx| BookmarksList::new(state.clone()));

    cx.new(|_cx| Self { state, add_btn, quick_btn, list })
  }
}

impl Render for BookmarksView {
  fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    div()
      .size_full()
      .p_3()
      .flex()
      .flex_col()
      .gap_y_3()
      .child(
        div()
          .flex()
          .items_center()
          .gap_x_2()
          .child(self.add_btn.clone())
          .child(self.quick_btn.clone()),
      )
      .child(div().flex_1().overflow_y_scrollbar().child(self.list.clone()))
  }
}
