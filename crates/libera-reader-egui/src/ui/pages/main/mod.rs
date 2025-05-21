use crate::router::Router;
use crate::ui::pages::main::content::Content;
use egui::Context;

pub(crate) mod content;
pub(crate) mod side_bar;
mod header;

pub(crate) struct Main {
  side_bar: side_bar::State,
  content: Content,
}
impl Main {
  pub fn new() -> Self {
    Self { side_bar: side_bar::State::new(), content: Content::new() }
  }
  pub(crate) fn make(&mut self, ctx: &Context, router: &mut Router) {
    self.side_bar.make(ctx, router);
    self.content.make(ctx, router);
  }
}
