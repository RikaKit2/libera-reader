use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LayoutMode {
  #[default]
  Continuous,
  PagedSingle,
  PagedDual,
}

impl LayoutMode {
  pub fn is_continuous(&self) -> bool {
    matches!(self, Self::Continuous)
  }

  pub fn toggle_continuous(&mut self) {
    *self = match self {
      Self::Continuous => Self::PagedSingle,
      Self::PagedSingle | Self::PagedDual => Self::Continuous,
    };
  }
}
