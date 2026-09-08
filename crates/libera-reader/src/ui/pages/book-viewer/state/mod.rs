pub mod layout_mode;
pub mod sidebar_tab;
pub mod tts_config;
pub mod zoom;

pub use layout_mode::LayoutMode;
pub use sidebar_tab::SidebarTab;
pub use tts_config::{TtsConfigProvider, TtsEngineConfig, TtsVoiceConfig};
pub use zoom::ZoomPreset;

use crate::db::models::books::book::BookPath;
use crate::types::HashMap;
use crate::ui::pages::book_viewer::document::DocumentData;
use gpui::SharedString;
use mutool::{BBox, OutlineNode, PageDimensions};
#[derive(Debug, Clone, PartialEq)]
pub struct OutlineItem {
  pub title: SharedString,
  pub page: usize,
  pub depth: usize,
  pub children: Vec<OutlineItem>,
}

impl From<OutlineNode> for OutlineItem {
  fn from(node: OutlineNode) -> Self {
    Self {
      title: SharedString::from(node.title),
      page: node.page,
      depth: node.depth,
      children: node.children.into_iter().map(OutlineItem::from).collect(),
    }
  }
}
#[derive(Debug, Clone, PartialEq)]
pub struct SearchHit {
  pub page: usize,
  pub text: SharedString,
  pub bbox: Option<BBox>,
}

#[derive(Clone)]
pub struct BookViewerState {
  pub current_book: Option<BookPath>,
  pub current_document: Option<DocumentData>,
  pub page_sizes: Vec<PageDimensions>,
  pub current_page: usize,
  pub total_pages: usize,
  pub title: SharedString,
  pub active_sidebar_tab: SidebarTab,
  pub layout_mode: LayoutMode,
  pub zoom_preset: ZoomPreset,
  pub zoom_factor: f32,
  pub invert_colors: bool,
  pub is_fullscreen: bool,

  // Selection state
  pub selection_handles: HashMap<usize, gpui_base::TextSelectionHandle>,
  pub selection_page: Option<usize>,
  pub selected_text: Option<String>,
  // Search state
  pub search_open: bool,
  pub search_query: String,
  pub search_results: Vec<SearchHit>,
  pub current_search_idx: usize,

  // Bookmark search state
  pub bookmark_search_query: String,
  // Document outline
  pub outline: Vec<OutlineItem>,

  // TTS State
  pub tts_playing: bool,
  pub tts_engine: SharedString,
  pub tts_voice: SharedString,
  pub tts_speed: f32,
  pub tts_pause_ms: u32,
  pub tts_auto_turn: bool,
}

impl Default for BookViewerState {
  fn default() -> Self {
    Self {
      current_book: None,
      current_document: None,
      page_sizes: Vec::new(),
      current_page: 1,
      total_pages: 1,
      title: SharedString::from(""),
      active_sidebar_tab: SidebarTab::None,
      layout_mode: LayoutMode::Continuous,
      zoom_preset: ZoomPreset::Percent100,
      zoom_factor: 1.0,
      invert_colors: false,
      is_fullscreen: false,
      selection_handles: HashMap::default(),
      selection_page: None,
      selected_text: None,

      search_open: false,
      search_query: String::new(),
      search_results: Vec::new(),
      current_search_idx: 0,

      bookmark_search_query: String::new(),
      outline: Vec::new(),

      tts_playing: false,
      tts_engine: SharedString::from("piper"),
      tts_voice: SharedString::from("ru_RU-irina-medium"),
      tts_speed: 1.0,
      tts_pause_ms: 150,
      tts_auto_turn: true,
    }
  }
}
impl std::fmt::Debug for BookViewerState {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("BookViewerState")
      .field("current_book", &self.current_book)
      .field("current_document", &self.current_document)
      .field("page_sizes", &self.page_sizes)
      .field("current_page", &self.current_page)
      .field("total_pages", &self.total_pages)
      .field("title", &self.title)
      .field("active_sidebar_tab", &self.active_sidebar_tab)
      .field("layout_mode", &self.layout_mode)
      .field("zoom_preset", &self.zoom_preset)
      .field("zoom_factor", &self.zoom_factor)
      .field("invert_colors", &self.invert_colors)
      .field("is_fullscreen", &self.is_fullscreen)
      .field("selection_page", &self.selection_page)
      .field("selected_text", &self.selected_text)
      .field("search_open", &self.search_open)
      .field("search_query", &self.search_query)
      .field("search_results", &self.search_results)
      .field("current_search_idx", &self.current_search_idx)
      .field("bookmark_search_query", &self.bookmark_search_query)
      .field("outline", &self.outline)
      .field("tts_playing", &self.tts_playing)
      .field("tts_engine", &self.tts_engine)
      .field("tts_voice", &self.tts_voice)
      .field("tts_speed", &self.tts_speed)
      .field("tts_pause_ms", &self.tts_pause_ms)
      .field("tts_auto_turn", &self.tts_auto_turn)
      .finish()
  }
}

impl BookViewerState {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn set_book(&mut self, path: BookPath, total_pages: usize, title: SharedString) {
    self.current_book = Some(path);
    self.total_pages = total_pages.max(1);
    self.current_page = 1;
    self.title = title;
    self.selected_text = None;
    self.selection_page = None;
    self.selection_handles.clear();
  }

  pub fn set_document(&mut self, doc: DocumentData, title: SharedString) {
    self.current_book = Some(doc.book_path.clone());
    self.total_pages = doc.total_pages.max(1);
    self.page_sizes = doc.page_sizes.clone();
    self.outline = doc.outline.iter().cloned().map(OutlineItem::from).collect();
    self.current_document = Some(doc);
    self.current_page = 1;
    self.title = title;
    self.selected_text = None;
    self.selection_page = None;
    self.selection_handles.clear();
  }

  pub fn get_or_create_selection_handle(
    &mut self, page: usize, window: &gpui::Window, cx: &mut gpui::App,
  ) -> gpui_base::TextSelectionHandle {
    self
      .selection_handles
      .entry(page)
      .or_insert_with(|| {
        let handle = gpui_base::TextSelectionHandle::new("", cx);
        let sub = handle.refresh_window_on_change(window, cx);
        sub.detach();
        handle
      })
      .clone()
  }

  pub fn clear_selection_handles(&mut self) {
    self.selection_handles.clear();
  }

  pub fn page_size(&self, page: usize) -> PageDimensions {
    if page == 0 || self.page_sizes.is_empty() {
      PageDimensions::default()
    } else {
      self.page_sizes.get(page - 1).copied().unwrap_or_default()
    }
  }

  pub fn next_page(&mut self) {
    if self.current_page < self.total_pages {
      self.current_page += 1;
    }
  }

  pub fn prev_page(&mut self) {
    if self.current_page > 1 {
      self.current_page -= 1;
    }
  }

  pub fn go_to_page(&mut self, page: usize) {
    self.current_page = page.clamp(1, self.total_pages);
  }

  pub fn toggle_sidebar_tab(&mut self, tab: SidebarTab) {
    self.active_sidebar_tab.toggle(tab);
  }

  pub fn toggle_invert_colors(&mut self) {
    self.invert_colors = !self.invert_colors;
  }

  pub fn toggle_search(&mut self) {
    self.search_open = !self.search_open;
    if !self.search_open {
      self.search_query.clear();
      self.search_results.clear();
      self.current_search_idx = 0;
    }
  }

  pub fn set_zoom(&mut self, preset: ZoomPreset) {
    self.zoom_preset = preset;
    if let Some(factor) = preset.factor() {
      self.zoom_factor = factor;
    }
  }

  pub fn set_bookmark_search_query(&mut self, query: String) {
    self.bookmark_search_query = query;
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::db::models::books::book::{BookDir, BookExt, BookPath};
  use mutool::{OutlineNode, PageDimensions};

  #[test]
  fn test_bookmark_search_query() {
    let mut state = BookViewerState::new();
    assert_eq!(state.bookmark_search_query, "");
    state.set_bookmark_search_query("chapter 1".to_string());
    assert_eq!(state.bookmark_search_query, "chapter 1");
  }

  #[test]
  fn test_page_navigation() {
    let mut state = BookViewerState::new();
    state.total_pages = 10;
    state.current_page = 1;

    state.next_page();
    assert_eq!(state.current_page, 2);

    state.prev_page();
    assert_eq!(state.current_page, 1);

    state.go_to_page(7);
    assert_eq!(state.current_page, 7);

    state.go_to_page(100);
    assert_eq!(state.current_page, 10);
  }

  #[test]
  fn test_zoom_preset_factor() {
    let mut state = BookViewerState::new();
    assert_eq!(state.zoom_factor, 1.0);

    state.set_zoom(ZoomPreset::Percent150);
    assert_eq!(state.zoom_factor, 1.5);

    state.set_zoom(ZoomPreset::Percent200);
    assert_eq!(state.zoom_factor, 2.0);
  }

  #[test]
  fn test_search_toggle_and_clear() {
    let mut state = BookViewerState::new();
    assert!(!state.search_open);

    state.toggle_search();
    assert!(state.search_open);
    state.search_query = "Rust".to_string();

    state.toggle_search();
    assert!(!state.search_open);
    assert!(state.search_query.is_empty());
  }

  #[test]
  fn test_document_data_setting() {
    let mut state = BookViewerState::new();
    let book_path = BookPath {
      parent_dir: BookDir::new(std::path::PathBuf::from("/books")),
      name: "test".into(),
      ext: BookExt::PDF("pdf".into()),
      deleted: false,
    };

    let outline = vec![OutlineNode::new("Intro".into(), 1, 0, None)];
    let page_sizes = vec![PageDimensions::new(600.0, 800.0), PageDimensions::new(600.0, 800.0)];

    let doc = DocumentData { book_path: book_path.clone(), total_pages: 2, page_sizes, outline };

    state.set_document(doc, "My Book".into());
    assert_eq!(state.total_pages, 2);
    assert_eq!(state.title, "My Book");
    assert_eq!(state.outline.len(), 1);
    assert_eq!(state.outline[0].title, "Intro");
    assert_eq!(state.page_size(1).width, 600.0);
    assert_eq!(state.page_size(1).height, 800.0);
  }
}
