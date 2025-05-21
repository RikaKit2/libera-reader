use crate::ui::pages::Pages;
use eframe::{run_native, Frame, NativeOptions};
use egui::Context;
use egui_extras::install_image_loaders;

mod router;
mod ui;

pub struct App {
  pages: Pages,
}

impl App {
  pub fn new() -> Self {
    Self { pages: Pages::new() }
  }
}

impl eframe::App for App {
  fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
    install_image_loaders(ctx);
    self.pages.make(ctx);
  }
}


fn main() -> eframe::Result {
  run_native("Libera Reader", NativeOptions::default(), Box::new(|_cc| Ok(Box::new(App::new()))))
}
