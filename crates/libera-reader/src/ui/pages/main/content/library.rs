use super::books_page::BooksPage;
use crate::books_state::TargetList;
use gpui::{AppContext, Context, Entity, IntoElement, Render, Window};

pub(crate) struct Library {
  page: Entity<BooksPage>,
}

impl Library {
  pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
    let page = cx.new(|cx| BooksPage::new(window, cx, TargetList::Library));
    Self { page }
  }
}

impl Render for Library {
  fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    self.page.clone()
  }
}
