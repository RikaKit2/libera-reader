use crate::ui::pages::main::content::{Bookmarks, Favorite, FileManager, History, Library, Settings, Stats};
use crate::ui::pages::main::side_bar::SideBar;
use gpui::{App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div};
use libera_reader_core::ctx::GlobalCTX;
use libera_reader_core::db::models::{RootRoute, Route};

pub(crate) mod content;
pub(crate) mod side_bar;

pub(crate) struct MainPage {
  side_bar: Entity<SideBar>,
  library: Entity<Library>,
  file_manager: Entity<FileManager>,
  history: Entity<History>,
  favorite: Entity<Favorite>,
  bookmarks: Entity<Bookmarks>,
  stats: Entity<Stats>,
  settings: Entity<Settings>,
}

impl MainPage {
  pub(crate) fn new(
    window: &mut Window, cx: &mut App, books_state: Entity<crate::books_state::BooksState>,
  ) -> Entity<Self> {
    let bookmarks = cx.new(|cx| Bookmarks::new(window, cx, books_state.clone()));
    let library = cx.new(|cx| Library::new(window, cx, books_state.clone()));
    let history = cx.new(|cx| History::new(window, cx, books_state.clone()));
    let favorite = cx.new(|cx| Favorite::new(window, cx, books_state.clone()));
    let settings = Settings::new(window, cx);

    cx.new(|c| Self {
      side_bar: SideBar::new(c),
      library,
      file_manager: FileManager::new(c),
      history,
      favorite,
      bookmarks,
      stats: Stats::new(c),
      settings,
    })
  }
}

impl Render for MainPage {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let curr_route = cx.ctx().settings.read().route;
    div().w_full().h_full().flex().children([
      div().w_12().h_full().child(self.side_bar.clone()),
      match curr_route {
        RootRoute::Main(route) => match route {
          Route::Library => div().w_full().h_full().child(self.library.clone()),
          Route::FileManager => div().w_full().h_full().child(self.file_manager.clone()),
          Route::History => div().w_full().h_full().child(self.history.clone()),
          Route::Favorite => div().w_full().h_full().child(self.favorite.clone()),
          Route::BookMarks => div().w_full().h_full().child(self.bookmarks.clone()),
          Route::Stats => div().w_full().h_full().child(self.stats.clone()),
          Route::Settings => div().w_full().h_full().child(self.settings.clone()),
        },
        _ => div(),
      },
    ])
  }
}
