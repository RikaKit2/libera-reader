use crate::app_ext::AppExt;
use crate::db::models::{RootRoute, Route};
use crate::ui::pages::main::content::{
  Bookmarks, Favorite, FileManager, History, Library, Settings, Stats,
};
use crate::ui::pages::main::side_bar::SideBar;
use gpui::{
  App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div,
};

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
  pub(crate) fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
    let bookmarks = cx.new(|cx| Bookmarks::new(window, cx));
    let library = cx.new(|cx| Library::new(window, cx));
    let history = cx.new(|cx| History::new(window, cx));
    let favorite = cx.new(|cx| Favorite::new(window, cx));
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
    let curr_route = cx.settings().read().route;
    div().w_full().h_full().flex().children([
      div().w_12().h_full().child(self.side_bar.clone()),
      match curr_route {
        RootRoute::Main(route) => {
          let content = match route {
            Route::Library => self.library.clone().into_any_element(),
            Route::FileManager => self.file_manager.clone().into_any_element(),
            Route::History => self.history.clone().into_any_element(),
            Route::Favorite => self.favorite.clone().into_any_element(),
            Route::BookMarks => self.bookmarks.clone().into_any_element(),
            Route::Stats => self.stats.clone().into_any_element(),
            Route::Settings => self.settings.clone().into_any_element(),
          };
          div().size_full().child(content)
        }
        _ => div(),
      },
    ])
  }
}
