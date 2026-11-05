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

/// Calculates zoom factor to fit content horizontally within available width.
/// Result is clamped between 0.25 (25%) and 4.0 (400%).
pub fn calculate_fit_width(available_width: f32, page_width: f32) -> f32 {
  if page_width <= 0.0 || available_width <= 0.0 {
    1.0
  } else {
    (available_width / page_width).clamp(0.25, 4.0)
  }
}

/// Calculates zoom factor to fit content vertically within available height.
/// Result is clamped between 0.25 (25%) and 4.0 (400%).
pub fn calculate_fit_page(available_height: f32, page_height: f32) -> f32 {
  if page_height <= 0.0 || available_height <= 0.0 {
    1.0
  } else {
    (available_height / page_height).clamp(0.25, 4.0)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use mutool::PageDimensions;

  #[test]
  fn test_fit_zoom_calculations() {
    let page_dims = PageDimensions::new(595.0, 842.0);
    let available_width = 1200.0;
    let available_height = 900.0;

    let width_factor = calculate_fit_width(available_width, page_dims.width);
    assert!((width_factor - (1200.0 / 595.0)).abs() < 0.01);

    let page_factor = calculate_fit_page(available_height, page_dims.height);
    assert!((page_factor - (900.0 / 842.0)).abs() < 0.01);
  }

  #[test]
  fn test_fit_zoom_clamp_and_edge_cases() {
    assert_eq!(calculate_fit_width(0.0, 595.0), 1.0);
    assert_eq!(calculate_fit_width(1200.0, 0.0), 1.0);
    assert_eq!(calculate_fit_width(100.0, 1000.0), 0.25); // Clamped to 0.25
    assert_eq!(calculate_fit_width(5000.0, 100.0), 4.0); // Clamped to 4.0

    assert_eq!(calculate_fit_page(0.0, 842.0), 1.0);
    assert_eq!(calculate_fit_page(900.0, 0.0), 1.0);
    assert_eq!(calculate_fit_page(100.0, 1000.0), 0.25);
    assert_eq!(calculate_fit_page(5000.0, 100.0), 4.0);
  }
}
