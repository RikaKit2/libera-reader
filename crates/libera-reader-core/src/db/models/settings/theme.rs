use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AppTheme {
  Sunset,
  Wireframe,
  Dark,
  Synthwave,
  Black,
  Luxury,
  Dracula,
  Night,
  Dim,
  AyuDark,
}

impl fmt::Display for AppTheme {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let theme_name = match self {
      Self::Sunset => "Sunset",
      Self::Wireframe => "Wireframe",
      Self::Dark => "Dark",
      Self::Synthwave => "Synthwave",
      Self::Black => "Black",
      Self::Luxury => "Luxury",
      Self::Dracula => "Dracula",
      Self::Night => "Night",
      Self::Dim => "Dim",
      Self::AyuDark => "Ayu Dark",
    };
    write!(f, "{}", theme_name)
  }
}
