use gpui::{AnyElement, App, IntoElement, ParentElement, SharedString, Styled, Window, div};
use gpui_component::{Icon, select::SelectItem};
use libera_reader_core::db::models::CardDisplayMode;
use libera_reader_core::db::models::books::book::{Book, BookSize};
use rust_i18n::t;
use std::fmt;

/// Lightweight in-memory representation of a book.
/// Instead of storing the heavy `Book` struct (which includes full path, user data, etc.),
/// we store only what's needed for display and sorting in the UI.
#[derive(Clone)]
pub struct LightBook {
  pub id: SharedString,
  pub name: SharedString,
  pub name_lower: SharedString,
  #[allow(dead_code)]
  pub ext: SharedString,
  pub ext_lower: SharedString,
  pub size: u64,
  pub last_opened: u64,
  pub is_favorite: bool,
  pub has_thumbnail: bool,
  /// String with `\u{200B}` (zero-width space) inserted between every character
  /// for pixel-perfect line wrapping in the UI.
  #[allow(dead_code)]
  pub formatted_title: SharedString,
  pub parent_dir: SharedString,
  pub deleted: bool,
  pub bookmark_count: usize,
}

impl LightBook {
  /// Build a LightBook from a full Book, generating formatted title once.
  pub fn from_book(book: &Book, has_thumbnail: bool) -> Self {
    let BookSize::BYTES(size) = book.book_size;
    let display_name = book.book_path.display_name();
    let formatted_title = Self::format_title_pixel_perfect(display_name.as_ref());
    let name = book.book_path.name.clone();
    let ext = book.book_path.ext.to_string();

    Self {
      id: book.id.clone().into(),
      name: name.clone(),
      name_lower: name.to_lowercase().into(),
      ext: ext.clone().into(),
      ext_lower: ext.to_lowercase().into(),
      size,
      last_opened: book.user_data.last_opened,
      is_favorite: book.user_data.favorite,
      has_thumbnail,
      formatted_title,
      parent_dir: book.parent_dir.clone().into(),
      deleted: book.book_path.deleted,
      bookmark_count: book.bookmarks.len(),
    }
  }

  fn format_title_pixel_perfect(title: &str) -> SharedString {
    let mut breakable_title = String::with_capacity(title.len() * 4);
    for ch in title.chars() {
      breakable_title.push(ch);
      breakable_title.push('\u{200B}');
    }
    breakable_title.into()
  }

  #[allow(dead_code)]
  pub fn get_mutool_error(
    &self, db: &libera_reader_core::db::DB,
  ) -> anyhow::Result<Option<mutool::mutool_error::MuToolError>> {
    if let Some(book) =
      db.get_book(libera_reader_core::db::models::books::book::BookPath::from_id(&self.id))?
    {
      book.get_mutool_error(db)
    } else {
      Ok(None)
    }
  }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum TargetList {
  Library,
  Favorites,
  History,
  Bookmarks,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SortField {
  Name,
  #[allow(dead_code)]
  Size,
  #[allow(dead_code)]
  Type,
  LastOpened,
}

impl SortField {
  #[allow(dead_code)]
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
#[allow(dead_code)]
pub struct DisplayModeOption(pub CardDisplayMode);

impl DisplayModeOption {
  #[allow(dead_code)]
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
