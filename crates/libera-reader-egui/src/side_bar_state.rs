use crate::router::MainRoute;

pub(crate) enum BtnType {
  Hovered,
  Active,
  Empty,
}

pub(crate) struct SideBarState {
  hovered_btn: Option<MainRoute>,
  active_btn: Option<MainRoute>,
}

impl SideBarState {
  pub fn new() -> Self {
    Self {
      hovered_btn: None,
      active_btn: None,
    }
  }
  pub fn get_btn_type(&self, main_route: &MainRoute) -> BtnType {
    if self.btn_is_active(main_route) {
      BtnType::Active
    } else if self.btn_is_hover(main_route) {
      BtnType::Hovered
    } else {
      BtnType::Empty
    }
  }
  fn btn_is_active(&self, main_route: &MainRoute) -> bool {
    if !self.active_btn.is_none() {
      &self.active_btn.as_ref().unwrap() == &main_route
    } else {
      false
    }
  }
  fn btn_is_hover(&self, main_route: &MainRoute) -> bool {
    if !self.hovered_btn.is_none() {
      &self.hovered_btn.as_ref().unwrap() == &main_route
    } else {
      false
    }
  }
  pub fn set_hovered_btn(&mut self, target_route: MainRoute) {
    self.hovered_btn = Some(target_route);
  }
  pub fn set_active_btn(&mut self, target_route: MainRoute) {
    self.active_btn = Some(target_route);
  }
}
impl Default for SideBarState {
  fn default() -> Self {
    SideBarState::new()
  }
}
