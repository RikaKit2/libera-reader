use super::books_page::BooksPage;
use crate::books_state::TargetList;
use gpui::{AppContext, Context, Entity, IntoElement, Render, Window};

pub(crate) struct History {
  page: Entity<BooksPage>,
}

impl History {
  pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
    let page = cx.new(|cx| BooksPage::new(window, cx, TargetList::History));
    Self { page }
  }
}

impl Render for History {
  fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    self.page.clone()
  }
}
