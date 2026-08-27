pub mod layout_mode;
pub mod sidebar_tab;
pub mod tts_config;
pub mod zoom;

pub use layout_mode::LayoutMode;
pub use sidebar_tab::SidebarTab;
pub use tts_config::{TtsConfigProvider, TtsEngineConfig, TtsVoiceConfig};
pub use zoom::ZoomPreset;

use crate::db::models::books::book::BookPath;
use gpui::SharedString;

#[derive(Debug, Clone)]
pub struct OutlineItem {
  pub title: SharedString,
  pub page: usize,
  pub depth: usize,
  pub children: Vec<OutlineItem>,
}

#[derive(Debug, Clone)]
pub struct SearchHit {
  pub page: usize,
  pub text: SharedString,
}

#[derive(Debug, Clone)]
pub struct BookViewerState {
  pub current_book: Option<BookPath>,
  pub current_page: usize,
  pub total_pages: usize,
  pub title: SharedString,
  pub active_sidebar_tab: SidebarTab,
  pub layout_mode: LayoutMode,
  pub zoom_preset: ZoomPreset,
  pub zoom_factor: f32,
  pub invert_colors: bool,
  pub is_fullscreen: bool,

  // Search state
  pub search_open: bool,
  pub search_query: String,
  pub search_results: Vec<SearchHit>,
  pub current_search_idx: usize,

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
      current_page: 1,
      total_pages: 1,
      title: SharedString::from(""),
      active_sidebar_tab: SidebarTab::None,
      layout_mode: LayoutMode::PagedSingle,
      zoom_preset: ZoomPreset::Percent100,
      zoom_factor: 1.0,
      invert_colors: false,
      is_fullscreen: false,

      search_open: false,
      search_query: String::new(),
      search_results: Vec::new(),
      current_search_idx: 0,

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

impl BookViewerState {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn set_book(&mut self, path: BookPath, total_pages: usize, title: SharedString) {
    self.current_book = Some(path);
    self.total_pages = total_pages.max(1);
    self.current_page = 1;
    self.title = title;
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
}
