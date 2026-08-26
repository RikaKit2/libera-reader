use crate::db::models::CardDisplayMode;
use gpui::{AnyElement, App, IntoElement, ParentElement, SharedString, Styled, Window, div};
use gpui_component::{Icon, select::SelectItem};
use rust_i18n::t;
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TargetList {
  Library,
  Favorites,
  History,
  Bookmarks,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SortField {
  Name,
  Size,
  Type,
  LastOpened,
}

impl SortField {
  pub fn available_for(target: TargetList) -> Vec<Self> {
    match target {
      TargetList::History => vec![Self::Name, Self::Size, Self::Type, Self::LastOpened],
      _ => vec![Self::Name, Self::Size, Self::Type],
    }
  }
}

impl fmt::Display for SortField {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      SortField::Name => write!(f, "Name"),
      SortField::Size => write!(f, "Size"),
      SortField::Type => write!(f, "Type"),
      SortField::LastOpened => write!(f, "Last Opened"),
    }
  }
}

impl SelectItem for SortField {
  type Value = SortField;
  fn title(&self) -> SharedString {
    match self {
      SortField::Name => t!("components.sort_dropdown.fields.name").to_string().into(),
      SortField::Size => t!("components.sort_dropdown.fields.size").to_string().into(),
      SortField::Type => t!("components.sort_dropdown.fields.type").to_string().into(),
      SortField::LastOpened => t!("components.sort_dropdown.fields.last_opened").to_string().into(),
    }
  }
  fn value(&self) -> &Self::Value {
    self
  }
}

#[derive(Clone, Copy)]
pub struct SortConfig {
  pub field: SortField,
  pub is_reversed: bool,
}

impl Default for SortConfig {
  fn default() -> Self {
    Self { field: SortField::Name, is_reversed: false }
  }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DisplayModeOption(pub CardDisplayMode);

impl DisplayModeOption {
  pub fn all() -> Vec<Self> {
    vec![
      Self(CardDisplayMode::Compact),
      Self(CardDisplayMode::Detailed),
      Self(CardDisplayMode::List),
    ]
  }
}

impl SelectItem for DisplayModeOption {
  type Value = CardDisplayMode;

  fn title(&self) -> SharedString {
    match self.0 {
      CardDisplayMode::Compact => t!("components.sort_dropdown.mode.compact").to_string().into(),
      CardDisplayMode::Detailed => t!("components.sort_dropdown.mode.detailed").to_string().into(),
      CardDisplayMode::List => t!("components.sort_dropdown.mode.list").to_string().into(),
    }
  }

  fn value(&self) -> &Self::Value {
    &self.0
  }

  fn display_title(&self) -> Option<AnyElement> {
    let icon_path = match self.0 {
      CardDisplayMode::Compact => "layout-grid.svg",
      CardDisplayMode::Detailed => "layout-dashboard.svg",
      CardDisplayMode::List => "layout-list.svg",
    };
    Some(Icon::new(Icon::empty()).path(icon_path).into_any_element())
  }

  fn render(&self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
    let icon_path = match self.0 {
      CardDisplayMode::Compact => "layout-grid.svg",
      CardDisplayMode::Detailed => "layout-dashboard.svg",
      CardDisplayMode::List => "layout-list.svg",
    };
    div()
      .w_full()
      .flex()
      .justify_center()
      .items_center()
      .py_1()
      .child(Icon::new(Icon::empty()).path(icon_path))
  }
}
