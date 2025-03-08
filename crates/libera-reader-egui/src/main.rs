use crate::router::{MainRoute, Route};
use eframe::{run_native, Frame, NativeOptions};
use egui::Context;
use egui_extras::install_image_loaders;

mod router;
mod side_bar;


pub struct App {
  route: Route,
  side_bar: side_bar::State,
}

impl App {
  pub fn new() -> Self {
    Self {
      route: Route::Main(MainRoute::Library),
      side_bar: Default::default(),
    }
  }
  fn change_route(&mut self, route: Route) { self.route = route; }
}

impl eframe::App for App {
  fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
    install_image_loaders(ctx);
    self.side_bar(ctx);
  }
}


fn main() -> eframe::Result {
  run_native("Libera Reader", NativeOptions::default(),
             Box::new(|_cc| Ok(Box::new(App::new()))))
}
