use gpui::{Pixels, px};

// ============================================================================
// Common Spacing & Radiuses
// ============================================================================

/// Small border radius (3px) used for buttons, inputs, outline items.
pub const RADIUS_SM: Pixels = px(3.0);
/// Medium border radius (4px) used for bookmark action buttons, bookmark items, TTS play button.
pub const RADIUS_MD: Pixels = px(4.0);

// ============================================================================
// Header Bar Constants
// ============================================================================

pub mod header {
  use super::*;

  /// Height of the top header bar (32px).
  pub const HEIGHT: Pixels = px(32.0);
  /// Horizontal padding of the header bar (4px).
  pub const PADDING_X: Pixels = px(4.0);

  /// Tight gap between header button groups (2px).
  pub const GAP_TIGHT: Pixels = px(2.0);
  /// Normal gap between center pagination items (4px).
  pub const GAP_NORMAL: Pixels = px(4.0);

  /// Standard square button size in header (28px).
  pub const BTN_SIZE: Pixels = px(28.0);

  /// Large icon size (22px).
  pub const ICON_SIZE_LG: Pixels = px(22.0);
  /// Medium icon size (20px).
  pub const ICON_SIZE_MD: Pixels = px(20.0);
  /// Small icon size (18px).
  pub const ICON_SIZE_SM: Pixels = px(18.0);

  /// Page number input box width (52px).
  pub const PAGE_INPUT_WIDTH: Pixels = px(52.0);
  /// Page number input box height (28px).
  pub const PAGE_INPUT_HEIGHT: Pixels = px(28.0);
  /// Page number input horizontal padding (4px).
  pub const PAGE_INPUT_PADDING_X: Pixels = px(4.0);

  /// Zoom preset select width (135px).
  pub const ZOOM_SELECT_WIDTH: Pixels = px(135.0);
  /// Zoom preset select height (28px).
  pub const ZOOM_SELECT_HEIGHT: Pixels = px(28.0);
  /// Zoom preset select horizontal padding (2px).
  pub const ZOOM_SELECT_PADDING_X: Pixels = px(2.0);
}

// ============================================================================
// Search Bar Constants
// ============================================================================

pub mod search_bar {
  use super::*;

  /// Height of the search bar container (34px).
  pub const HEIGHT: Pixels = px(34.0);
  /// Horizontal padding of the search bar container (8px).
  pub const PADDING_X: Pixels = px(8.0);
  /// Gap between elements in the search bar (4px).
  pub const GAP_X: Pixels = px(4.0);
  /// Search input height (26px).
  pub const INPUT_HEIGHT: Pixels = px(26.0);
  /// Search input horizontal padding (3px).
  pub const INPUT_PADDING_X: Pixels = px(3.0);

  /// Search prev/next button height (26px).
  pub const BTN_HEIGHT: Pixels = px(26.0);
  /// Search prev/next button horizontal padding (10px).
  pub const BTN_PADDING_X: Pixels = px(10.0);
  /// Search button internal gap between icon and text (4px).
  pub const BTN_GAP_X: Pixels = px(4.0);
  /// Search prev/next button icon size (16px).
  pub const BTN_ICON_SIZE: Pixels = px(16.0);

  /// Search close button square size (26px).
  pub const CLOSE_BTN_SIZE: Pixels = px(26.0);
  /// Search close button icon size (18px).
  pub const CLOSE_ICON_SIZE: Pixels = px(18.0);

  /// Search results counter horizontal padding (6px).
  pub const COUNTER_PADDING_X: Pixels = px(6.0);
}

// ============================================================================
// Sidebar Constants
// ============================================================================

pub mod sidebar {
  use super::*;

  /// Default sidebar width (250px).
  pub const WIDTH: Pixels = px(250.0);
  /// Sidebar header bar height (32px).
  pub const HEADER_HEIGHT: Pixels = px(32.0);
  /// Sidebar header horizontal padding (8px).
  pub const HEADER_PADDING_X: Pixels = px(8.0);
}

// ============================================================================
// Bookmarks Sub-module Constants
// ============================================================================

pub mod bookmarks {
  use super::*;

  /// Bookmarks view container padding (10px).
  pub const CONTAINER_PADDING: Pixels = px(10.0);
  /// Bookmarks view vertical gap (10px).
  pub const CONTAINER_GAP_Y: Pixels = px(10.0);
  /// Gap between Add and Quick action buttons (8px).
  pub const CONTAINER_HEADER_GAP_X: Pixels = px(8.0);

  /// Bookmark action button height (30px).
  pub const ACTION_BTN_HEIGHT: Pixels = px(30.0);
  /// Bookmark action button internal gap (6px).
  pub const ACTION_BTN_GAP_X: Pixels = px(6.0);
  /// Bookmark action button icon size (14px).
  pub const ACTION_BTN_ICON_SIZE: Pixels = px(14.0);

  /// Gap between items in the bookmarks list (6px).
  pub const LIST_GAP_Y: Pixels = px(6.0);

  /// Empty state vertical padding (36px).
  pub const EMPTY_PY: Pixels = px(36.0);
  /// Empty state horizontal padding (8px).
  pub const EMPTY_PX: Pixels = px(8.0);
  /// Empty state vertical gap (8px).
  pub const EMPTY_GAP_Y: Pixels = px(8.0);
  /// Empty state icon size (32px).
  pub const EMPTY_ICON_SIZE: Pixels = px(32.0);

  /// Bookmark item padding (8px).
  pub const ITEM_PADDING: Pixels = px(8.0);
  /// Bookmark item horizontal gap (8px).
  pub const ITEM_GAP_X: Pixels = px(8.0);
  /// Bookmark item icon size (16px).
  pub const ITEM_ICON_SIZE: Pixels = px(16.0);

  /// Bookmark item delete button size (22px).
  pub const DEL_BTN_SIZE: Pixels = px(22.0);
  /// Bookmark item delete button icon size (14px).
  pub const DEL_BTN_ICON_SIZE: Pixels = px(14.0);
}

// ============================================================================
// Outline Sub-module Constants
// ============================================================================

pub mod outline {
  use super::*;

  /// Outline tree item vertical padding (2px).
  pub const ITEM_PY: Pixels = px(2.0);
  /// Outline tree item horizontal padding (4px).
  pub const ITEM_PX: Pixels = px(4.0);
  /// Outline tree item horizontal gap (4px).
  pub const ITEM_GAP_X: Pixels = px(4.0);

  /// Outline tree toggle chevron container size (16px).
  pub const TOGGLE_SIZE: Pixels = px(16.0);
  /// Outline tree toggle chevron icon size (12px).
  pub const TOGGLE_ICON_SIZE: Pixels = px(12.0);

  /// Outline child branch left indentation (16px).
  pub const CHILDREN_INDENT: Pixels = px(16.0);
}

// ============================================================================
// Thumbnails Sub-module Constants
// ============================================================================

pub mod thumbnails {
  use super::*;

  /// Thumbnail card width (100px).
  pub const CARD_WIDTH: Pixels = px(100.0);
  /// Thumbnail card height (140px).
  pub const CARD_HEIGHT: Pixels = px(140.0);
}

// ============================================================================
// TTS Sub-module Constants
// ============================================================================

pub mod tts {
  use super::*;

  /// TTS main play button size (32px).
  pub const PLAY_BTN_SIZE: Pixels = px(32.0);
  /// TTS main play button icon size (20px).
  pub const PLAY_ICON_SIZE: Pixels = px(20.0);

  /// TTS prev/next navigation button size (28px).
  pub const NAV_BTN_SIZE: Pixels = px(28.0);
  /// TTS prev/next navigation button icon size (20px).
  pub const NAV_ICON_SIZE: Pixels = px(20.0);

  /// TTS reset settings button vertical padding (4px).
  pub const RESET_BTN_PY: Pixels = px(4.0);
  /// TTS reset settings button horizontal padding (8px).
  pub const RESET_BTN_PX: Pixels = px(8.0);

  /// TTS pause interval input box width (64px).
  pub const PAUSE_INPUT_WIDTH: Pixels = px(64.0);
  /// TTS pause interval input box height (26px).
  pub const PAUSE_INPUT_HEIGHT: Pixels = px(26.0);
  /// TTS pause interval input horizontal padding (4px).
  pub const PAUSE_INPUT_PX: Pixels = px(4.0);
  /// TTS pause section vertical gap (4px).
  pub const PAUSE_GAP_Y: Pixels = px(4.0);
  /// TTS pause row horizontal gap (6px).
  pub const PAUSE_GAP_X: Pixels = px(6.0);

  /// TTS speed slider track height (6px).
  pub const SPEED_TRACK_HEIGHT: Pixels = px(6.0);
}

// ============================================================================
// Viewport & Page Constants
// ============================================================================

pub mod viewport {
  /// Base standard page width in points (595.0 pt / A4 width).
  pub const PAGE_BASE_WIDTH: f32 = 595.0;
  /// Base standard page height in points (842.0 pt / A4 height).
  pub const PAGE_BASE_HEIGHT: f32 = 842.0;
}
