use crate::router::{BaseRoute, RootRoute, Router};
use eframe::{run_native, Frame, NativeOptions};
use egui::Context;
use egui_extras::install_image_loaders;
use ui::pages::main::side_bar;


mod router;
mod ui;

pub struct App {
  side_bar: side_bar::State,
  router: Router,
}

impl App {
  pub fn new() -> Self {
    let router = Router::new(RootRoute::Base(BaseRoute::Library));
    let side_bar = side_bar::State::new();
    Self { side_bar, router }
  }
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
