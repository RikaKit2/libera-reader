use gpui::SharedString;
use gpui_component::select::SelectItem;
use rust_i18n::t;
use serde::{Deserialize, Serialize};
pub enum ZoomMode {
  Fixed(f32),
  FitWidth,
  FitPage,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZoomPreset {
  Percent50,
  Percent75,
  Percent100,
  Percent125,
  Percent150,
  Percent200,
  FitWidth,
  FitPage,
}

impl ZoomPreset {
  pub fn all() -> Vec<Self> {
    vec![
      Self::Percent50,
      Self::Percent75,
      Self::Percent100,
      Self::Percent125,
      Self::Percent150,
      Self::Percent200,
      Self::FitWidth,
      Self::FitPage,
    ]
  }

  pub fn label(&self) -> String {
    match self {
      Self::Percent50 => "50%".to_string(),
      Self::Percent75 => "75%".to_string(),
      Self::Percent100 => "100%".to_string(),
      Self::Percent125 => "125%".to_string(),
      Self::Percent150 => "150%".to_string(),
      Self::Percent200 => "200%".to_string(),
      Self::FitWidth => t!("components.book_viewer.zoom.fit_width").to_string(),
      Self::FitPage => t!("components.book_viewer.zoom.fit_page").to_string(),
    }
  }

  pub fn factor(&self) -> Option<f32> {
    match self {
      Self::Percent50 => Some(0.5),
      Self::Percent75 => Some(0.75),
      Self::Percent100 => Some(1.0),
      Self::Percent125 => Some(1.25),
      Self::Percent150 => Some(1.5),
      Self::Percent200 => Some(2.0),
      Self::FitWidth | Self::FitPage => None,
    }
  }
}

impl SelectItem for ZoomPreset {
  type Value = ZoomPreset;

  fn title(&self) -> SharedString {
    self.label().into()
  }

  fn value(&self) -> &Self::Value {
    self
  }
}
