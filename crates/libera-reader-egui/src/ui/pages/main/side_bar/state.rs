use crate::glob::ROUTER;
use crate::router::{BaseRoute, RootRoute};
use crate::ui::pages::main::side_bar::{BORDER_ACTIVE_COLOR, BORDER_BASE_COLOR,
                                       BTN_ACTIVE_COLOR, BTN_BASE_COLOR, BTN_HOVER_COLOR};
use egui::Color32;
use std::cmp::PartialEq;

#[derive(Debug)]
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

pub(crate) struct BtnColorData {
  pub(crate) icon: Color32,
  pub(crate) strip: Color32,
}
impl BtnColorData {
  pub(crate) fn new(icon: Color32, strip: Color32) -> Self { Self { icon, strip } }
}

pub(crate) struct Btn {
  pub(crate) color_data: BtnColorData,
  action: BtnAction,
  btn_route: BaseRoute,
}

impl Btn {
  pub(crate) fn new(btn_route: BaseRoute) -> Self {
    Self { color_data: Self::get_data_of_base_btn(), action: BtnAction::None, btn_route }
  }
  pub(crate) fn set_btn_action_as_none(&mut self) {
    self.color_data = Self::get_data_of_base_btn();
    self.action = BtnAction::None;
    println!("as none: {:?}", &self.btn_route);
  }
  pub(crate) fn set_btn_action_as_hover(&mut self) {
    self.color_data = Self::get_data_of_hovered_btn();
    self.action = BtnAction::Hover;
  }
  pub(crate) fn set_btn_action_as_click(&mut self) {
    ROUTER.write().unwrap().change_route(RootRoute::Base(self.btn_route.clone()));
    self.color_data = Self::get_data_of_clicked_btn();
    self.action = BtnAction::Click;
  }
  fn get_data_of_clicked_btn() -> BtnColorData {
    BtnColorData::new(*BTN_ACTIVE_COLOR, *BORDER_ACTIVE_COLOR)
  }
  fn get_data_of_hovered_btn() -> BtnColorData {
    BtnColorData::new(*BTN_HOVER_COLOR, *BORDER_BASE_COLOR)
  }
  fn get_data_of_base_btn() -> BtnColorData {
    BtnColorData::new(*BTN_BASE_COLOR, *BORDER_BASE_COLOR)
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
    match &ROUTER.read().unwrap().inn {
      RootRoute::Base(curr_route) => {
        inn.apply_action_to_btn(BtnAction::Click, curr_route);
      }
      _ => {}
    };
    inn
  }

  pub fn apply_action_to_btn(&mut self, action: BtnAction, target_route: &BaseRoute) {
    match &action {
      BtnAction::Click => {
        // если target_route это новая активная кнопка
        // для отсутствия повторной активации активной кнопки
        if !ROUTER.read().unwrap().compare_with_root_route(target_route) {
          self.btns.iter_mut().for_each(|btn| btn.set_btn_action_as_none());

          let target_btn: &mut Btn = self.get_mut_btn_by_route(target_route);
          target_btn.set_btn_action_as_click()
        }
      }
      BtnAction::Hover => {
        let target_btn: &mut Btn = self.get_mut_btn_by_route(target_route);
        if &target_btn.action == &BtnAction::None {
          target_btn.set_btn_action_as_hover();
        }
      }
      BtnAction::None => {
        let target_btn: &mut Btn = self.get_mut_btn_by_route(target_route);
        if &target_btn.action == &BtnAction::Hover {
          target_btn.set_btn_action_as_none();
        }
      }
    };
  }
  fn get_mut_btn_by_route(&mut self, target_route: &BaseRoute) -> &mut Btn {
    self.btns.iter_mut().filter(|btn| btn.btn_route == *target_route).last().unwrap()
  }
  pub fn get_btn_by_route(&self, target_route: &BaseRoute) -> &Btn {
    self.btns.iter().filter(|btn| btn.btn_route == *target_route).last().unwrap()
  }
}
