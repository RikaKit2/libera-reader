use gpui::*;

actions!(
  book_viewer,
  [
    NextPage,
    PrevPage,
    FirstPage,
    LastPage,
    ZoomIn,
    ZoomOut,
    ResetZoom,
    FitWidth,
    FitPage,
    ToggleSearch,
    NextSearchMatch,
    PrevSearchMatch,
    CloseSearch,
    ToggleFullscreen,
    ToggleInvertColors,
    ToggleSidebar,
    ToggleBookmarks,
    ToggleOutline,
    ToggleThumbnails,
    ToggleTts,
    TtsPlayPause,
    TtsNext,
    TtsPrev,
    ExitViewer,
  ]
);

pub const BOOK_VIEWER_CONTEXT: &str = "BookViewer";

pub fn default_keybindings() -> Vec<KeyBinding> {
  vec![
    // Navigation
    KeyBinding::new("right", NextPage, None),
    KeyBinding::new("pagedown", NextPage, None),
    KeyBinding::new("space", NextPage, None),
    KeyBinding::new("left", PrevPage, None),
    KeyBinding::new("pageup", PrevPage, None),
    KeyBinding::new("shift-space", PrevPage, None),
    KeyBinding::new("home", FirstPage, None),
    KeyBinding::new("ctrl-home", FirstPage, None),
    KeyBinding::new("end", LastPage, None),
    KeyBinding::new("ctrl-end", LastPage, None),
    // Search
    KeyBinding::new("ctrl-f", ToggleSearch, None),
    KeyBinding::new("f3", NextSearchMatch, None),
    KeyBinding::new("shift-f3", PrevSearchMatch, None),
    KeyBinding::new("escape", CloseSearch, None),
    // Zoom
    KeyBinding::new("ctrl-=", ZoomIn, None),
    KeyBinding::new("ctrl-+", ZoomIn, None),
    KeyBinding::new("ctrl--", ZoomOut, None),
    KeyBinding::new("ctrl-0", ResetZoom, None),
    KeyBinding::new("ctrl-w", FitWidth, None),
    KeyBinding::new("ctrl-shift-p", FitPage, None),
    // View & Presentation
    KeyBinding::new("f11", ToggleFullscreen, None),
    KeyBinding::new("ctrl-shift-f", ToggleFullscreen, None),
    KeyBinding::new("ctrl-i", ToggleInvertColors, None),
    KeyBinding::new("ctrl-b", ToggleSidebar, None),
    // Sidebar tabs
    KeyBinding::new("alt-1", ToggleBookmarks, None),
    KeyBinding::new("alt-2", ToggleOutline, None),
    KeyBinding::new("alt-3", ToggleThumbnails, None),
    KeyBinding::new("alt-4", ToggleTts, None),
    // Audio / TTS
    KeyBinding::new("ctrl-space", TtsPlayPause, None),
    KeyBinding::new("ctrl-alt-right", TtsNext, None),
    KeyBinding::new("ctrl-alt-left", TtsPrev, None),
  ]
}
