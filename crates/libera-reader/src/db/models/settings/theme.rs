use gpui::SharedString;
use gpui_component::select::SelectItem;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum AppTheme {
  Sunset,
  Dark,
  Synthwave,
  Black,
  Luxury,
  Dracula,
  Night,
  Dim,
  AyuDark,
  EverforestDark,
}
impl AppTheme {
  pub fn all() -> Vec<Self> {
    vec![
      Self::Sunset,
      Self::Dark,
      Self::Synthwave,
      Self::Black,
      Self::Luxury,
      Self::Dracula,
      Self::Night,
      Self::Dim,
      Self::AyuDark,
      Self::EverforestDark,
    ]
  }
}

impl fmt::Display for AppTheme {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let theme_name = match self {
      Self::Sunset => "Sunset",
      Self::Dark => "Dark",
      Self::Synthwave => "Synthwave",
      Self::Black => "Black",
      Self::Luxury => "Luxury",
      Self::Dracula => "Dracula",
      Self::Night => "Night",
      Self::Dim => "Dim",
      Self::AyuDark => "Ayu Dark",
      Self::EverforestDark => "Everforest Dark",
    };
    write!(f, "{}", theme_name)
  }
}

impl SelectItem for AppTheme {
  type Value = AppTheme;

  fn title(&self) -> SharedString {
    self.to_string().into()
  }

  fn value(&self) -> &Self::Value {
    self
  }
}
