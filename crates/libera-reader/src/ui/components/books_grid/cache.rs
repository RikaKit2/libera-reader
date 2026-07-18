use gpui::{App, RenderImage, SharedString};
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Arc;

#[derive(Clone)]
pub enum CoverState {
  Loading,
  Loaded(Arc<RenderImage>), // Working with hardware textures
  Failed,
}

/// LRU cache of decoded cover textures, keyed by **book id** (not by grid
/// position). Keying by id keeps a cover attached to its book across reorders
/// (reverse / sort / search), so swapping the keys list never shows a stale
/// cover from the cell that used to occupy that position.
pub struct BoundedCache {
  pub inner: LruCache<SharedString, CoverState>,
  evicted: Vec<Arc<RenderImage>>,
}

impl BoundedCache {
  pub fn new(max_cached: usize) -> Self {
    Self { inner: LruCache::new(NonZeroUsize::new(max_cached).unwrap()), evicted: Vec::new() }
  }

  pub fn get_mut(&mut self, id: &SharedString) -> Option<CoverState> {
    self.inner.get(id).cloned()
  }

  pub fn insert(&mut self, id: SharedString, state: CoverState) {
    if let Some((_, evicted_state)) = self.inner.push(id, state)
      && let CoverState::Loaded(img) = evicted_state
    {
      self.evicted.push(img);
    }
  }

  /// Drop a `Loading` placeholder if present (used by the loader to cancel
  /// stale requests for books that scrolled out of view).
  pub fn pop_loading(&mut self, id: &SharedString) {
    if let Some(CoverState::Loading) = self.inner.peek(id) {
      self.inner.pop(id);
    }
  }

  // ⚡️ MIGRATION: Accept AppContext and explicitly remove tile from GPU atlas
  pub fn flush_evictions(&mut self, cx: &mut App) {
    let evicted = std::mem::take(&mut self.evicted);
    for img in evicted {
      cx.drop_image(img, None);
    }
  }
}
