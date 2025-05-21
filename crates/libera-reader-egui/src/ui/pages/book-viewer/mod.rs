use egui::Context;

pub(crate) struct BookViewer {}
impl BookViewer {
  pub(crate) fn new() -> Self {
    Self {}
  }
  pub(crate) fn make(&mut self, ctx: &Context) {}
}