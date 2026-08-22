use crate::db::models::CardDisplayMode;
use crate::db::models::books::book::{Book, BookSize, BookSnapshot};
use gpui::{AnyElement, App, IntoElement, ParentElement, SharedString, Styled, Window, div};
use gpui_component::{Icon, select::SelectItem};
use rust_i18n::t;
use std::fmt;

/// Lightweight in-memory representation of a book.
/// Instead of storing the heavy `Book` struct (which includes full path, user data, etc.),
/// we store only what's needed for display and sorting in the UI.
///
/// All UI-facing strings are pre-computed once in `from_book` to avoid per-frame
/// allocations in `render()` closures (see documentation/ram.md §6).
#[derive(Clone)]
pub struct LightBook {
  pub id: SharedString,
  pub name: SharedString,
  pub name_lower: SharedString,
  #[allow(dead_code)]
  pub ext: SharedString,
  pub ext_lower: SharedString,
  /// Upper-cased extension, pre-computed for display in card footer.
  #[allow(dead_code)]
  pub ext_upper: SharedString,
  pub size: u64,
  /// Pre-formatted "File size: N MB" label for the card footer.
  pub size_label: SharedString,
  /// Pre-computed "<format_label>: <EXT>" label for the card footer.
  pub format_label: SharedString,
  /// Stable GPUI ElementId prefixes, pre-built to avoid `format!()` in render closures.
  pub cover_btn_id: SharedString,
  pub fav_btn_id: SharedString,
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
    let ext_upper = ext.to_uppercase();

    // Pre-compute UI labels once. These are zero-allocation clones in render().
    let format_label = format!("{}: {}", t!("components.card.format_label"), ext_upper).into();
    let size_label = format!("File size: {} MB", size / 1_048_576).into();
    let cover_btn_id = format!("cover_{}", book.id).into();
    let fav_btn_id = format!("fav_{}", book.id).into();

    Self {
      id: book.id.clone().into(),
      name: name.clone(),
      name_lower: name.to_lowercase().into(),
      ext: ext.clone().into(),
      ext_lower: ext.to_lowercase().into(),
      ext_upper: ext_upper.into(),
      size,
      size_label,
      format_label,
      cover_btn_id,
      fav_btn_id,
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

  /// Build a `LightBook` from a UI-ready `BookSnapshot`.
  ///
  /// This is the post-channel path: services construct snapshots from `Book`s
  /// at the boundary, and the UI side never sees the heavy `Book` form. All
  /// UI-facing strings (labels, stable element ids) are pre-computed here so
  /// the `render()` closures stay allocation-free (see documentation/ram.md §6).
  pub fn from_snapshot(snapshot: &BookSnapshot, has_thumbnail: bool) -> Self {
    let formatted_title = Self::format_title_pixel_perfect(&snapshot.name);
    let ext_upper = snapshot.ext.to_uppercase();

    let format_label = format!("{}: {}", t!("components.card.format_label"), ext_upper).into();
    let size_label = format!("File size: {} MB", snapshot.size / 1_048_576).into();
    let cover_btn_id = format!("cover_{}", snapshot.id).into();
    let fav_btn_id = format!("fav_{}", snapshot.id).into();

    Self {
      id: snapshot.id.clone().into(),
      name: snapshot.name.clone().into(),
      name_lower: snapshot.name.to_lowercase().into(),
      ext: snapshot.ext.clone().into(),
      ext_lower: snapshot.ext.to_lowercase().into(),
      ext_upper: ext_upper.into(),
      size: snapshot.size,
      size_label,
      format_label,
      cover_btn_id,
      fav_btn_id,
      last_opened: snapshot.last_opened,
      is_favorite: snapshot.is_favorite,
      has_thumbnail,
      formatted_title,
      parent_dir: snapshot.parent_dir.clone().into(),
      deleted: snapshot.deleted,
      bookmark_count: snapshot.bookmark_count,
    }
  }

  #[allow(dead_code)]
  pub fn get_mutool_error(
    &self, db: &crate::db::DB,
  ) -> anyhow::Result<Option<mutool::mutool_error::MuToolError>> {
    if let Some(book) = db.get_book(crate::db::models::books::book::BookPath::from_id(&self.id))? {
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
