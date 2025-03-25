use crate::router::BaseRoute;
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
  pub icon: Color32,
  pub strip: Color32,
  action: BtnAction,
  btn_route: BaseRoute,
}

impl Btn {
  pub fn new(btn_route: BaseRoute) -> Self {
    Self { icon: *BTN_BASE_COLOR, strip: *BORDER_BASE_COLOR, action: BtnAction::None, btn_route }
  }
  pub fn set_status(&mut self, status: BtnAction) {
    match status {
      BtnAction::Click => {
        if self.action != BtnAction::Click {
          self.strip = *BORDER_ACTIVE_COLOR;
          self.icon = *BTN_ACTIVE_COLOR;
          self.action = BtnAction::Click;
        }
      }
      BtnAction::Hover => {
        if self.action != BtnAction::Click {
          self.action = BtnAction::Hover;
          self.icon = *BTN_HOVER_COLOR;
          self.strip = *BORDER_BASE_COLOR;
        }
      }
      BtnAction::None => {
        self.icon = *BTN_BASE_COLOR;
        self.strip = *BORDER_BASE_COLOR;
        self.action = BtnAction::None;
      }
    }
  }
}

pub(crate) struct State {
  btns: [Btn; 7],
}

impl State {
  pub fn new() -> Self {
    let mut inn = Self {
      btns: [
        Btn::new(BaseRoute::Library), Btn::new(BaseRoute::FileManager),
        Btn::new(BaseRoute::History), Btn::new(BaseRoute::Favorite),
        Btn::new(BaseRoute::BookMarks), Btn::new(BaseRoute::Stats),
        Btn::new(BaseRoute::Settings)
      ]
    };
    inn
  }
  pub fn change_btn_status(&mut self, status: BtnAction, route: &BaseRoute) {
    self.get_mut_btn_by_route(route).set_status(status);
  }
  fn get_mut_btn_by_route(&mut self, target_route: &BaseRoute) -> &mut Btn {
    self.btns.iter_mut().filter(|btn| btn.btn_route == *target_route).last().unwrap()
  }
  pub(crate) fn get_btn_by_route(&self, target_route: &BaseRoute) -> &Btn {
    self.btns.iter().filter(|btn| btn.btn_route == *target_route).last().unwrap()
  }
}
