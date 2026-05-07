use gpui::SharedString;
use gpui_component::select::SelectItem;
use rust_i18n::t;
use std::fmt;

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

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum TargetList {
  Library,
  Favorites,
  History,
  Bookmarks,
}
