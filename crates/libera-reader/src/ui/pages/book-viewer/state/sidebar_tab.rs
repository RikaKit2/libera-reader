use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SidebarTab {
  #[default]
  None,
  Outline,
  Bookmarks,
  Thumbnails,
  Tts,
}

impl SidebarTab {
  pub fn is_open(&self) -> bool {
    !matches!(self, Self::None)
  }

  pub fn toggle(&mut self, tab: Self) {
    if *self == tab {
      *self = Self::None;
    } else {
      *self = tab;
    }
  }
}
