use crate::ui::pages::CTX;
use gpui::prelude::*;
use gpui::{App, Entity};
use std::sync::Arc;

pub(crate) struct BookViewer {
  ctx: Arc<CTX>,
}
impl BookViewer {
  pub(crate) fn new(cx: &mut App, ctx: Arc<CTX>) -> Entity<Self> {
    cx.new(|_| Self { ctx })
  }
}
