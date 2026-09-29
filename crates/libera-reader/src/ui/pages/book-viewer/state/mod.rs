pub mod layout_mode;
pub mod sidebar_tab;
pub mod tts_config;
pub mod zoom;

pub use layout_mode::LayoutMode;
pub use sidebar_tab::SidebarTab;
pub use tts_config::{TtsConfigProvider, TtsEngineConfig, TtsVoiceConfig};
pub use zoom::{ZoomPreset, calculate_fit_page, calculate_fit_width};

use crate::db::models::books::book::BookPath;
use crate::types::HashMap;
use crate::ui::pages::book_viewer::document::DocumentData;
use gpui::{Pixels, SharedString, Size};
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
  pub is_searching: bool,
  pub has_searched: bool,
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
      is_searching: false,
      has_searched: false,
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
      .field("is_searching", &self.is_searching)
      .field("has_searched", &self.has_searched)
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
      self.clear_search();
    }
  }

  pub fn clear_search(&mut self) {
    self.search_query.clear();
    self.search_results.clear();
    self.current_search_idx = 0;
    self.is_searching = false;
    self.has_searched = false;
  }

  pub fn set_search_results(&mut self, results: Vec<SearchHit>) {
    self.is_searching = false;
    self.has_searched = true;
    self.search_results = results;
    if !self.search_results.is_empty() {
      let start_idx =
        self.search_results.iter().position(|hit| hit.page >= self.current_page).unwrap_or(0);
      self.current_search_idx = start_idx;
      let page = self.search_results[self.current_search_idx].page;
      self.go_to_page(page);
    } else {
      self.current_search_idx = 0;
    }
  }

  pub fn next_search_match(&mut self) {
    if !self.search_results.is_empty() {
      self.current_search_idx = (self.current_search_idx + 1) % self.search_results.len();
      let page = self.search_results[self.current_search_idx].page;
      self.go_to_page(page);
    }
  }

  pub fn prev_search_match(&mut self) {
    if !self.search_results.is_empty() {
      if self.current_search_idx == 0 {
        self.current_search_idx = self.search_results.len().saturating_sub(1);
      } else {
        self.current_search_idx -= 1;
      }
      let page = self.search_results[self.current_search_idx].page;
      self.go_to_page(page);
    }
  }

  pub fn set_zoom(&mut self, preset: ZoomPreset) {
    self.zoom_preset = preset;
    if let Some(factor) = preset.factor() {
      self.zoom_factor = factor;
    }
  }

  pub fn update_fit_zoom(&mut self, available_size: Size<Pixels>) {
    let page = self.page_size(self.current_page);
    match self.zoom_preset {
      ZoomPreset::FitWidth => {
        let w = f32::from(available_size.width) - 48.0; // padding + scrollbar
        self.zoom_factor = calculate_fit_width(w, page.width);
      }
      ZoomPreset::FitPage => {
        let h = f32::from(available_size.height) - 64.0; // padding + header + gap
        self.zoom_factor = calculate_fit_page(h, page.height);
      }
      _ => {}
    }
  }

  pub fn zoom_in(&mut self) {
    let presets = ZoomPreset::all();
    let current_idx = presets.iter().position(|p| *p == self.zoom_preset).unwrap_or(2);
    if current_idx + 1 < presets.len() && presets[current_idx + 1].factor().is_some() {
      self.set_zoom(presets[current_idx + 1]);
    } else {
      self.adjust_zoom_by(0.15);
    }
  }

  pub fn zoom_out(&mut self) {
    let presets = ZoomPreset::all();
    let current_idx = presets.iter().position(|p| *p == self.zoom_preset).unwrap_or(2);
    if current_idx > 0 && presets[current_idx - 1].factor().is_some() {
      self.set_zoom(presets[current_idx - 1]);
    } else {
      self.adjust_zoom_by(-0.15);
    }
  }

  pub fn adjust_zoom_by(&mut self, delta: f32) {
    let new_factor = (self.zoom_factor + delta).clamp(0.25, 4.0);
    self.zoom_factor = new_factor;
  }

  pub fn toggle_fullscreen(&mut self, window: &gpui::Window) {
    self.is_fullscreen = !self.is_fullscreen;
    window.toggle_fullscreen();
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
  fn test_zoom_in_and_out() {
    let mut state = BookViewerState::new();
    state.set_zoom(ZoomPreset::Percent100);
    assert_eq!(state.zoom_factor, 1.0);

    state.zoom_in();
    assert_eq!(state.zoom_preset, ZoomPreset::Percent125);
    assert_eq!(state.zoom_factor, 1.25);

    state.zoom_in();
    assert_eq!(state.zoom_preset, ZoomPreset::Percent150);
    assert_eq!(state.zoom_factor, 1.5);

    state.zoom_out();
    assert_eq!(state.zoom_preset, ZoomPreset::Percent125);
    assert_eq!(state.zoom_factor, 1.25);

    state.adjust_zoom_by(0.10);
    assert!((state.zoom_factor - 1.35).abs() < 0.001);
  }

  #[test]
  fn test_update_fit_zoom() {
    let mut state = BookViewerState::new();
    state.page_sizes = vec![PageDimensions::new(600.0, 800.0)];
    state.current_page = 1;

    state.set_zoom(ZoomPreset::FitWidth);
    state.update_fit_zoom(Size { width: gpui::px(1248.0), height: gpui::px(900.0) });
    // available w = 1248.0 - 48.0 = 1200.0. page.width = 600.0. factor = 2.0.
    assert!((state.zoom_factor - 2.0).abs() < 0.01);

    state.set_zoom(ZoomPreset::FitPage);
    state.update_fit_zoom(Size { width: gpui::px(1200.0), height: gpui::px(864.0) });
    // available h = 864.0 - 64.0 = 800.0. page.height = 800.0. factor = 1.0.
    assert!((state.zoom_factor - 1.0).abs() < 0.01);
  }

  #[test]
  fn test_search_results_navigation_and_cycling() {
    let mut state = BookViewerState::new();
    state.total_pages = 10;
    state.current_page = 3;

    let hits = vec![
      SearchHit { page: 2, text: "hit on 2".into(), bbox: None },
      SearchHit { page: 4, text: "hit on 4".into(), bbox: None },
      SearchHit { page: 8, text: "hit on 8".into(), bbox: None },
    ];

    // Setting results from page 3 starts at the first match >= 3, which is hit on page 4 (idx 1)
    state.set_search_results(hits);
    assert_eq!(state.current_search_idx, 1);
    assert_eq!(state.current_page, 4);

    // Next match cycles forward to page 8
    state.next_search_match();
    assert_eq!(state.current_search_idx, 2);
    assert_eq!(state.current_page, 8);

    // Next match wraps around to page 2
    state.next_search_match();
    assert_eq!(state.current_search_idx, 0);
    assert_eq!(state.current_page, 2);

    // Prev match wraps backwards to page 8
    state.prev_search_match();
    assert_eq!(state.current_search_idx, 2);
    assert_eq!(state.current_page, 8);
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
