use crate::ui::pages::main::content::{BookMarks, Favorite, FileManager, History, Library, SettingsPage, Stats};
use crate::ui::pages::main::side_bar::SideBar;
use gpui::{div, App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window};
use libera_reader_core::ctx::GlobalCTX;
use libera_reader_core::db::models::{RootRoute, Route};

pub(crate) mod content;
pub(crate) mod side_bar;
mod header;

pub(crate) struct MainPage {
  side_bar: Entity<SideBar>,
  library: Entity<Library>,
  file_manager: Entity<FileManager>,
  history: Entity<History>,
  favorite: Entity<Favorite>,
  book_marks: Entity<BookMarks>,
  stats: Entity<Stats>,
  settings: Entity<SettingsPage>,
}

impl MainPage {
  pub(crate) fn new(cx: &mut App) -> Entity<Self> {
    cx.new(|c|
      Self {
        side_bar: SideBar::new(c),
        library: Library::new(c),
        file_manager: FileManager::new(c),
        history: History::new(c),
        favorite: Favorite::new(c),
        book_marks: BookMarks::new(c),
        stats: Stats::new(c),
        settings: SettingsPage::new(c),
      }
    )
  }
}

impl Render for MainPage {
  fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let curr_route = cx.ctx().settings.read().unwrap().route.clone();
    div().w_full().h_full().flex().children([
      div().w_12().h_full().child(self.side_bar.clone()),
      match curr_route {
        RootRoute::Main(route) => {
          match route {
            Route::Library => { div().w_full().h_full().child(self.library.clone()) }
            Route::FileManager => { div().w_full().h_full().child(self.file_manager.clone()) }
            Route::History => { div().w_full().h_full().child(self.history.clone()) }
            Route::Favorite => { div().w_full().h_full().child(self.favorite.clone()) }
            Route::BookMarks => { div().w_full().h_full().child(self.book_marks.clone()) }
            Route::Stats => { div().w_full().h_full().child(self.stats.clone()) }
            Route::Settings => { div().w_full().h_full().child(self.settings.clone()) }
          }
        }
        _ => { div() }
      }
    ])
  }
}
