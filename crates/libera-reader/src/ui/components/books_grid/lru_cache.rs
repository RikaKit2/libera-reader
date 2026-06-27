use gpui::*;
use libera_reader_core::db::DB;
use libera_reader_core::db::models::books::book::Book;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

/// An ImageCache that limits the number of cached images in memory.
/// It also caches the absence of thumbnails (None) to avoid redundant DB reads.
/// Evicted images are queued and removed from GPUI's asset cache during rendering.
pub struct LruImageCache {
  inner: HashMap<SharedString, Option<Arc<Image>>>,
  order: VecDeque<SharedString>,
  evicted_images: Vec<Arc<Image>>,
  max_images: usize,
}

impl LruImageCache {
  pub fn new(max_images: usize) -> Self {
    LruImageCache {
      inner: HashMap::new(),
      order: VecDeque::new(),
      evicted_images: Vec::new(),
      max_images,
    }
  }

  pub fn get(&mut self, id: &SharedString, db: &DB) -> Option<Arc<Image>> {
    if let Some(cached) = self.inner.get(id) {
      // Move to back of LRU order
      if let Some(pos) = self.order.iter().position(|x| x == id) {
        self.order.remove(pos);
        self.order.push_back(id.clone());
      }
      return cached.clone();
    }

    // Query DB (only once per book until evicted)
    let opt_image = if let Ok(Some(bytes)) = db.rt(|r| {
      if let Some(book) = r.get().primary::<Book>(id.to_string())? {
        book.get_thumbnail_data_in_txn(r)
      } else {
        Ok(None)
      }
    }) {
      Some(Arc::new(Image::from_bytes(ImageFormat::Jpeg, bytes)))
    } else {
      None
    };

    // Evict oldest if at capacity
    while self.inner.len() >= self.max_images {
      if let Some(oldest) = self.order.pop_front()
        && let Some(Some(img)) = self.inner.remove(&oldest)
      {
        self.evicted_images.push(img);
      }
    }

    self.inner.insert(id.clone(), opt_image.clone());
    self.order.push_back(id.clone());
    opt_image
  }

  /// Force clear/evict a specific book from the cache (e.g. when thumbnail was extracted)
  pub fn remove(&mut self, id: &SharedString) {
    if let Some(Some(img)) = self.inner.remove(id) {
      self.evicted_images.push(img);
    }
    if let Some(pos) = self.order.iter().position(|x| x == id) {
      self.order.remove(pos);
    }
  }

  /// Explicitly evict images from GPUI asset cache to free RAM
  pub fn flush_evictions(&mut self, cx: &mut App) {
    for img in std::mem::take(&mut self.evicted_images) {
      img.remove_asset(cx);
    }
  }
}
