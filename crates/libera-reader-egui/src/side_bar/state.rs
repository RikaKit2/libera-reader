use crate::router::MainRoute;
use crate::side_bar::{BORDER_ACTIVE_COLOR, BORDER_BASE_COLOR, BTN_ACTIVE_COLOR, BTN_BASE_COLOR, BTN_HOVER_COLOR};
use egui::Color32;

pub enum BtnStatus {
  Clicked,
  Hovered,
  Base,
}

pub struct BtnData {
  pub icon: Color32,
  pub strip: Color32,
}

impl BtnData {
  pub fn new() -> Self {
    Self { icon: *BTN_BASE_COLOR, strip: *BORDER_BASE_COLOR }
  }
  pub fn set_status(&mut self, status: BtnStatus) {
    match status {
      BtnStatus::Clicked => {
        self.strip = *BORDER_ACTIVE_COLOR;
        self.icon = *BTN_ACTIVE_COLOR;
      }
      BtnStatus::Hovered => { self.icon = *BTN_HOVER_COLOR; }
      BtnStatus::Base => {
        self.icon = *BTN_BASE_COLOR;
        self.strip = *BORDER_BASE_COLOR;
      }
    }
  }
}
pub struct Btns {
  pub library: BtnData,
  pub file_manager: BtnData,
  pub history: BtnData,
  pub favorite: BtnData,
  pub bookmarks: BtnData,
  pub state: BtnData,
  pub settings: BtnData,
}
impl Btns {
  pub fn new() -> Self {
    Self {
      library: BtnData::new(),
      file_manager: BtnData::new(),
      history: BtnData::new(),
      favorite: BtnData::new(),
      bookmarks: BtnData::new(),
      state: BtnData::new(),
      settings: BtnData::new(),
    }
  }
  pub fn change_btn_status(&mut self, status: BtnStatus, route: &MainRoute) {
    match route {
      MainRoute::Library => { self.library.set_status(status) }
      MainRoute::FileManager => { self.file_manager.set_status(status) }
      MainRoute::History => { self.history.set_status(status) }
      MainRoute::Favorite => { self.favorite.set_status(status) }
      MainRoute::BookMarks => { self.bookmarks.set_status(status) }
      MainRoute::Stats => { self.state.set_status(status) }
      MainRoute::Settings => { self.settings.set_status(status) }
    }
  }
  pub fn get_btn_data_by_route(&self, route: &MainRoute) -> &BtnData {
    match route {
      MainRoute::Library => { &self.library }
      MainRoute::FileManager => { &self.file_manager }
      MainRoute::History => { &self.history }
      MainRoute::Favorite => { &self.favorite }
      MainRoute::BookMarks => { &self.bookmarks }
      MainRoute::Stats => { &self.state }
      MainRoute::Settings => { &self.settings }
    }
  }
}
pub(crate) struct State {
  pub btn_colors: Btns,
}

impl State {
  pub fn new() -> Self {
    Self { btn_colors: Btns::new() }
  }
}
impl Default for State {
  fn default() -> Self {
    State::new()
  }
}
