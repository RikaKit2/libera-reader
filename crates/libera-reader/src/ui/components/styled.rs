use gpui::Pixels;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Copy, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub enum Size {
  Pixels(Pixels),
  XSmall,
  Small,
  #[default]
  Medium,
  Large,
}

impl From<Pixels> for Size {
  fn from(size: Pixels) -> Self {
    Size::Pixels(size)
  }
}
