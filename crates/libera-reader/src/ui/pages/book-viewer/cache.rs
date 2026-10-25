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
  images: LruCache<usize, (Arc<RenderImage>, u32)>,
  loading_dpis: crate::types::HashMap<usize, u32>,
  failed_images: crate::types::HashMap<usize, String>,
  texts: LruCache<usize, PageTextState>,
  links: LruCache<usize, PageLinksState>,
  evicted: Vec<Arc<RenderImage>>,
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
      loading_dpis: crate::types::HashMap::default(),
      failed_images: crate::types::HashMap::default(),
      texts: LruCache::new(text_cap),
      links: LruCache::new(links_cap),
      evicted: Vec::new(),
    }
  }

  pub fn ensure_min_capacity(&mut self, min_cap: usize) {
    let target = min_cap.max(MAX_CACHED_PAGE_IMAGES);
    if target > self.images.cap().get() {
      self.images.resize(NonZeroUsize::new(target).unwrap());
    }
  }

  pub fn get_image(&mut self, page: usize) -> Option<PageImageState> {
    if let Some((img, _)) = self.images.get(&page) {
      return Some(PageImageState::Loaded(img.clone()));
    }
    if self.loading_dpis.contains_key(&page) {
      return Some(PageImageState::Loading);
    }
    if let Some(err) = self.failed_images.get(&page) {
      return Some(PageImageState::Failed(err.clone()));
    }
    None
  }

  pub fn insert_loaded_image(&mut self, page: usize, img: Arc<RenderImage>, dpi: u32) {
    self.loading_dpis.swap_remove(&page);
    self.failed_images.swap_remove(&page);
    if let Some((_, (evicted_img, _))) = self.images.push(page, (img, dpi)) {
      self.evicted.push(evicted_img);
    }
  }

  pub fn insert_image(&mut self, page: usize, state: PageImageState) {
    match state {
      PageImageState::Loaded(img) => {
        let dpi = self.loading_dpis.swap_remove(&page).unwrap_or(150);
        self.insert_loaded_image(page, img, dpi);
      }
      PageImageState::Loading => {
        self.loading_dpis.entry(page).or_insert(150);
      }
      PageImageState::Failed(err) => {
        self.loading_dpis.swap_remove(&page);
        self.failed_images.insert(page, err);
      }
      PageImageState::Unloaded => {
        self.loading_dpis.swap_remove(&page);
        self.failed_images.swap_remove(&page);
        if let Some((img, _)) = self.images.pop(&page) {
          self.evicted.push(img);
        }
      }
    }
  }

  pub fn flush_evictions(&mut self, cx: &mut gpui::App) {
    let evicted = std::mem::take(&mut self.evicted);
    for img in evicted {
      cx.drop_image(img, None);
    }
  }

  pub fn get_image_dpi(&mut self, page: usize) -> Option<u32> {
    self.images.peek(&page).map(|(_, dpi)| *dpi)
  }

  pub fn set_image_dpi(&mut self, page: usize, dpi: u32) {
    if let Some(entry) = self.images.peek_mut(&page) {
      entry.1 = dpi;
    }
  }

  pub fn get_loading_dpi(&self, page: usize) -> Option<u32> {
    self.loading_dpis.get(&page).copied()
  }

  pub fn set_loading_dpi(&mut self, page: usize, dpi: u32) {
    self.loading_dpis.insert(page, dpi);
  }

  pub fn pop_image_loading(&mut self, page: usize) {
    self.loading_dpis.swap_remove(&page);
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
    for (_, (img, _)) in self.images.iter() {
      self.evicted.push(img.clone());
    }
    self.images.clear();
    self.loading_dpis.clear();
    self.failed_images.clear();
    self.texts.clear();
    self.links.clear();
  }

  pub fn clear_images(&mut self) {
    for (_, (img, _)) in self.images.iter() {
      self.evicted.push(img.clone());
    }
    self.images.clear();
    self.loading_dpis.clear();
    self.failed_images.clear();
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

    cache.set_loading_dpi(1, 144);
    assert_eq!(cache.get_loading_dpi(1), Some(144));

    cache.pop_image_loading(1);
    assert!(cache.get_image(1).is_none());
    assert_eq!(cache.get_loading_dpi(1), None);

    cache.insert_text(1, PageTextState::Loading);
    assert!(matches!(cache.get_text(1), Some(PageTextState::Loading)));

    cache.set_loading_dpi(2, 72);
    cache.clear();
    assert!(cache.get_text(1).is_none());
    assert_eq!(cache.get_loading_dpi(2), None);
  }
}
