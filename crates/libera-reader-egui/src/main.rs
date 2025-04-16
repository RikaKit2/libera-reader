use crate::router::{RootRoute, Route, Router};
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
    let mut router = Router::new(RootRoute::Base(Route::Library));
    let mut side_bar = side_bar::State::new();
    side_bar.get_mut_btn(&Route::Library).mark_as_clicked(&mut router);
    Self { side_bar, router }
  }
}

impl eframe::App for App {
  fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
    install_image_loaders(ctx);
    self.make_side_bar(ctx);
    self.make_content(ctx);
  }
}


fn main() -> eframe::Result {
  run_native("Libera Reader", NativeOptions::default(), Box::new(|_cc| Ok(Box::new(App::new()))))
}
