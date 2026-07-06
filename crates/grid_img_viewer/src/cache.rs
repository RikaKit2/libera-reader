use gpui::{App, RenderImage};
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Arc;

#[derive(Clone)]
pub enum CoverState {
  Loading,
  Loaded(Arc<RenderImage>), // Working with hardware textures
  Failed,
}

pub struct BoundedCache {
  pub inner: LruCache<usize, CoverState>,
  evicted: Vec<Arc<RenderImage>>,
}

impl BoundedCache {
  pub fn new(max_cached: usize) -> Self {
    Self { inner: LruCache::new(NonZeroUsize::new(max_cached).unwrap()), evicted: Vec::new() }
  }

  pub fn get_mut(&mut self, idx: usize) -> Option<CoverState> {
    self.inner.get(&idx).cloned()
  }

  pub fn insert(&mut self, idx: usize, state: CoverState) {
    if let Some((_, evicted_state)) = self.inner.push(idx, state)
      && let CoverState::Loaded(img) = evicted_state
    {
      self.evicted.push(img);
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
