use serde::{Deserialize, Serialize};

/// Display mode for the reader viewport.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LayoutMode {
  /// Vertical continuous virtualized scroll list of pages
  #[default]
  Continuous,
  /// Single centered page view with page flipping
  PagedSingle,
  /// Two-page spread side-by-side
  PagedDual,
}

impl LayoutMode {
  /// Check if continuous scroll mode is active.
  pub fn is_continuous(&self) -> bool {
    matches!(self, Self::Continuous)
  }

  /// Toggle between continuous ribbon and single page mode.
  pub fn toggle_continuous(&mut self) {
    *self = match self {
      Self::Continuous => Self::PagedSingle,
      Self::PagedSingle | Self::PagedDual => Self::Continuous,
    };
  }
}
