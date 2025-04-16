use crate::App;
use egui::Context;
use egui::{CentralPanel, ScrollArea};

impl App {
  pub fn make_content(&mut self, ctx: &Context) {
    CentralPanel::default().show(ctx, |ui| {
      let mut style = (*ctx.style()).clone();
      ui.checkbox(&mut style.debug.debug_on_hover, "Debug on hover");
      ctx.set_style(style);
      ScrollArea::vertical().show(ui, |ui| {
        ui.set_width(ui.available_width());

      });
    });
  }
}
