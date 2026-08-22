use crate::app_ext::AppExt;
use crate::ui::pages::main::MainPage;
use crate::ui::pages::setup::SetupPage;
use gpui::{
  App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div,
};
#[path = "book-viewer/mod.rs"]
pub(crate) mod book_viewer;
pub(crate) mod main;
pub(crate) mod setup;

pub struct Pages {
  main_page: Entity<MainPage>,
  setup_page: Entity<SetupPage>,
}

impl Pages {
  pub fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
    cx.new(|c| Self { main_page: MainPage::new(window, c), setup_page: SetupPage::new(window, c) })
  }
}

impl Render for Pages {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let path_to_scan = cx.settings().read().path_to_scan.is_some();
    let setup_status = cx.settings().read().setup_is_done;
    match path_to_scan && setup_status {
      true => div().w_full().h_full().child(self.main_page.clone()),
      false => div().w_full().h_full().child(self.setup_page.clone()),
    }
  }
}
