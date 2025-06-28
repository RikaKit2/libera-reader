use crate::ui::pages::CTX;
use gpui::prelude::*;
use gpui::{App, Entity};
use std::sync::Arc;

#[allow(dead_code)]
pub(crate) struct BookViewer {
  ctx: Arc<CTX>,
}
impl BookViewer {
  #[allow(dead_code)]
  pub(crate) fn new(cx: &mut App, ctx: Arc<CTX>) -> Entity<Self> {
    cx.new(|_| Self { ctx })
  }
}
