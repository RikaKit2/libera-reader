use egui::{CentralPanel, Context, ScrollArea};
use crate::router::Router;

pub(crate) struct Content {}
impl Content {
  pub(crate) fn new() -> Self { Self {} }
  pub(crate) fn make(&self, ctx: &Context, router: &mut Router) {
    CentralPanel::default().show(ctx, |ui| {
      let mut style = (*ctx.style()).clone();
      ui.checkbox(&mut style.debug.debug_on_hover, "Debug on hover");
      ctx.set_style(style);
      ui.label(format!("{:?}", router));
      
      ScrollArea::vertical().show(ui, |ui| {
        ui.set_width(ui.available_width());
      });
    });
  }
}