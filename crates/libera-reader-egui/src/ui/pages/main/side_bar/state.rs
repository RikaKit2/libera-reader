use crate::router::{RootRoute, Route, Router};
use crate::side_bar::{BORDER_ACTIVE_COLOR, BORDER_BASE_COLOR, BTN_ACTIVE_COLOR, BTN_BASE_COLOR, BTN_HOVER_COLOR};
use egui::Color32;


pub enum BtnAction {
  Click,
  Hover,
  None,
}

impl PartialEq for BtnAction {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (BtnAction::Click, BtnAction::Click) => true,
      (BtnAction::Hover, BtnAction::Hover) => true,
      (BtnAction::None, BtnAction::None) => true,
      _ => false,
    }
  }
}

pub(crate) struct Btn {
  pub(crate) icon: Color32,
  pub(crate) strip: Color32,
  action: BtnAction,
  route: Route,
}

impl Btn {
  pub fn new(btn_route: Route) -> Self {
    Self { icon: *BTN_BASE_COLOR, strip: *BORDER_BASE_COLOR, action: BtnAction::None, route: btn_route }
  }
  pub(crate) fn mark_as_clicked(&mut self, router: &mut Router) {
    if self.action != BtnAction::Click {
      router.change_route(RootRoute::Base(self.route.clone()));
      self.strip = *BORDER_ACTIVE_COLOR;
      self.icon = *BTN_ACTIVE_COLOR;
      self.action = BtnAction::Click;
    }
  }
  pub(crate) fn mark_as_hovered(&mut self) {
    if self.action != BtnAction::Click {
      self.action = BtnAction::Hover;
      self.icon = *BTN_HOVER_COLOR;
      self.strip = *BORDER_BASE_COLOR;
    }
  }
  pub(crate) fn remove_mark(&mut self, router: &mut Router) {
    if !router.compare_with_root_route(&self.route) {
      self.icon = *BTN_BASE_COLOR;
      self.strip = *BORDER_BASE_COLOR;
      self.action = BtnAction::None;
    }
  }
}

pub(crate) struct State {
  btns: [Btn; 7],
}

impl State {
  pub fn new() -> Self {
    Self {
      btns: [
        Btn::new(Route::Library), Btn::new(Route::FileManager),
        Btn::new(Route::History), Btn::new(Route::Favorite),
        Btn::new(Route::BookMarks), Btn::new(Route::Stats),
        Btn::new(Route::Settings)
      ]
    }
  }
  pub(crate) fn get_mut_btn(&mut self, target_route: &Route) -> &mut Btn {
    self.btns.iter_mut().filter(|btn| btn.route == *target_route).last().unwrap()
  }
  pub(crate) fn get_btn(&self, target_route: &Route) -> &Btn {
    self.btns.iter().filter(|btn| btn.route == *target_route).last().unwrap()
  }
}
