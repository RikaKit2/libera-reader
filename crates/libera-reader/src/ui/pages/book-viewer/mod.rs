use gpui::prelude::*;
use gpui::{App, Entity};

#[allow(dead_code)]
pub(crate) struct BookViewer {}
impl BookViewer {
  #[allow(dead_code)]
  pub(crate) fn new(cx: &mut App) -> Entity<Self> {
    cx.new(|_| Self {})
  }
}
