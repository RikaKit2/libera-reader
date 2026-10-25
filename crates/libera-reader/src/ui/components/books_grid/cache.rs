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
  textures: LruCache<SharedString, Arc<RenderImage>>,
  in_flight: crate::types::HashSet<SharedString>,
  failed: crate::types::HashSet<SharedString>,
  evicted: Vec<Arc<RenderImage>>,
}

impl BoundedCache {
  pub fn new(max_cached: usize) -> Self {
    Self {
      textures: LruCache::new(NonZeroUsize::new(max_cached.max(16)).unwrap()),
      in_flight: crate::types::HashSet::default(),
      failed: crate::types::HashSet::default(),
      evicted: Vec::new(),
    }
  }

  /// Гарантирует, что размер LRU-кэша не меньше `min_cap` (число видимых карточек + буфер)
  /// и синхронизирует размер с настройками пользователя.
  pub fn sync_capacity(&mut self, desired_cap: usize) {
    let target = desired_cap.max(16);
    if self.textures.cap().get() != target {
      while self.textures.len() > target {
        if let Some((_, img)) = self.textures.pop_lru() {
          self.evicted.push(img);
        }
      }
      self.textures.resize(NonZeroUsize::new(target).unwrap());
    }
  }

  pub fn get_mut(&mut self, id: &SharedString) -> Option<CoverState> {
    if let Some(img) = self.textures.get(id) {
      return Some(CoverState::Loaded(img.clone()));
    }
    if self.in_flight.contains(id) {
      return Some(CoverState::Loading);
    }
    if self.failed.contains(id) {
      return Some(CoverState::Failed);
    }
    None
  }

  pub fn insert(&mut self, id: SharedString, state: CoverState) {
    match state {
      CoverState::Loading => {
        self.in_flight.insert(id);
      }
      CoverState::Loaded(img) => {
        self.in_flight.swap_remove(&id);
        self.failed.swap_remove(&id);
        if let Some((_, evicted_img)) = self.textures.push(id, img) {
          self.evicted.push(evicted_img);
        }
      }
      CoverState::Failed => {
        self.in_flight.swap_remove(&id);
        self.failed.insert(id);
      }
    }
  }

  pub fn pop_loading(&mut self, id: &SharedString) {
    self.in_flight.swap_remove(id);
  }

  pub fn flush_evictions(&mut self, cx: &mut App) {
    let evicted = std::mem::take(&mut self.evicted);
    for img in evicted {
      cx.drop_image(img, None);
    }
  }
}
