use crate::ui::pages::book_viewer::constants::viewport::{
  MAX_CACHED_PAGE_IMAGES, MAX_CACHED_TEXT_LAYERS,
};
use gpui::RenderImage;
use lru::LruCache;
use mutool::{PageLink, PageStructuredText};
use std::num::NonZeroUsize;
use std::sync::Arc;

/// Cached state of a rendered page bitmap.
#[derive(Clone, Default)]
pub enum PageImageState {
  #[default]
  Unloaded,
  Loading,
  Loaded(Arc<RenderImage>),
  Failed(String),
}

/// Cached state of structured text extracted from a page.
#[derive(Clone, Default)]
pub enum PageTextState {
  #[default]
  Unloaded,
  Loading,
  Loaded(Arc<PageStructuredText>),
  Failed(String),
}

/// Cached state of hyperlinks extracted from a page.
#[derive(Clone, Default)]
pub enum PageLinksState {
  #[default]
  Unloaded,
  Loading,
  Loaded(Arc<Vec<PageLink>>),
  Failed(String),
}

/// In-memory LRU cache storing recently viewed page images, text layers, and links.
pub struct BookViewerCache {
  images: LruCache<usize, PageImageState>,
  texts: LruCache<usize, PageTextState>,
  links: LruCache<usize, PageLinksState>,
}

impl Default for BookViewerCache {
  fn default() -> Self {
    Self::new()
  }
}

impl BookViewerCache {
  pub fn new() -> Self {
    let img_cap = NonZeroUsize::new(MAX_CACHED_PAGE_IMAGES).unwrap();
    let text_cap = NonZeroUsize::new(MAX_CACHED_TEXT_LAYERS).unwrap();
    let links_cap = NonZeroUsize::new(MAX_CACHED_TEXT_LAYERS).unwrap();

    Self {
      images: LruCache::new(img_cap),
      texts: LruCache::new(text_cap),
      links: LruCache::new(links_cap),
    }
  }

  pub fn get_image(&mut self, page: usize) -> Option<&PageImageState> {
    self.images.get(&page)
  }

  pub fn insert_image(&mut self, page: usize, state: PageImageState) {
    self.images.put(page, state);
  }

  pub fn pop_image_loading(&mut self, page: usize) {
    if let Some(PageImageState::Loading) = self.images.peek(&page) {
      self.images.pop(&page);
    }
  }

  pub fn get_text(&mut self, page: usize) -> Option<&PageTextState> {
    self.texts.get(&page)
  }

  pub fn insert_text(&mut self, page: usize, state: PageTextState) {
    self.texts.put(page, state);
  }

  pub fn pop_text_loading(&mut self, page: usize) {
    if let Some(PageTextState::Loading) = self.texts.peek(&page) {
      self.texts.pop(&page);
    }
  }

  pub fn get_links(&mut self, page: usize) -> Option<&PageLinksState> {
    self.links.get(&page)
  }

  pub fn insert_links(&mut self, page: usize, state: PageLinksState) {
    self.links.put(page, state);
  }

  pub fn clear(&mut self) {
    self.images.clear();
    self.texts.clear();
    self.links.clear();
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_cache_insert_and_eviction() {
    let mut cache = BookViewerCache::new();

    assert!(cache.get_image(1).is_none());
    cache.insert_image(1, PageImageState::Loading);
    assert!(matches!(cache.get_image(1), Some(PageImageState::Loading)));

    cache.pop_image_loading(1);
    assert!(cache.get_image(1).is_none());

    cache.insert_text(1, PageTextState::Loading);
    assert!(matches!(cache.get_text(1), Some(PageTextState::Loading)));

    cache.clear();
    assert!(cache.get_text(1).is_none());
  }
}
