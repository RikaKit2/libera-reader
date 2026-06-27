use crate::ui::pages::setup::SetupPage;
use crate::{books_state::BooksState, ui::pages::main::MainPage};
use gpui::{
  App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div,
};
use libera_reader_core::ctx::GlobalCTX;
use libera_reader_core::db::models::books::book::Book;
use libera_reader_core::types::LibraryEvent;
use tokio::sync::broadcast::Receiver;

#[path = "book-viewer/mod.rs"]
pub(crate) mod book_viewer;
pub(crate) mod main;
pub(crate) mod setup;

pub(crate) struct Pages {
  main_page: Entity<MainPage>,
  setup_page: Entity<SetupPage>,
}

impl Pages {
  pub fn new(
    window: &mut Window, cx: &mut App, initial_books: Vec<Book>, event_rx: Receiver<LibraryEvent>,
  ) -> Entity<Self> {
    let books_state = cx.new(|cx| BooksState::new(initial_books, event_rx, cx));
    cx.new(|c| Self {
      main_page: MainPage::new(window, c, books_state.clone()),
      setup_page: SetupPage::new(window, c),
    })
  }
}

impl Render for Pages {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let path_to_scan = cx.ctx().settings.read().path_to_scan.is_some();
    let setup_status = cx.ctx().settings.read().setup_is_done;
    match path_to_scan && setup_status {
      true => div().w_full().h_full().child(self.main_page.clone()),
      false => div().w_full().h_full().child(self.setup_page.clone()),
    }
  }
}
