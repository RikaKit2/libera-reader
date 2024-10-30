use crate::router::{MainRoute, Route};
use crate::side_bar_state::SideBarState;
use eframe::{run_native, Frame, NativeOptions};
use egui::Context;
use egui_extras::install_image_loaders;

mod sidebar;
mod router;
mod side_bar_state;

pub struct App {
  route: Route,
  side_bar_state: SideBarState,
}

impl App {
  pub fn new() -> Self {
    Self {
      route: Route::Main(MainRoute::Library),
      side_bar_state: Default::default(),
    }
  }
}

impl eframe::App for App {
  fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
    install_image_loaders(ctx);
    self.sidebar(ctx);
  }
}


fn main() -> eframe::Result {
  run_native("Libera Reader", NativeOptions::default(),
             Box::new(|_cc| Ok(Box::new(App::new()))))
}
