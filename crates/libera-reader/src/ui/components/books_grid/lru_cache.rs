use futures_util::FutureExt;
use futures_util::future::Shared;
use gpui::*;
use libera_reader_core::types::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;

/// An ImageCache that limits the number of cached images.
/// When the limit is exceeded, the oldest (least recently used) image is evicted.
/// This prevents memory bloat from thousands of decoded thumbnails.
pub struct LruImageCache {
  inner: HashMap<u64, ImageCacheItem>,
  order: VecDeque<u64>,
  max_images: usize,
}

impl LruImageCache {
  pub fn new(cx: &mut App, max_images: usize) -> Entity<Self> {
    let e =
      cx.new(|_cx| LruImageCache { inner: HashMap::default(), order: VecDeque::new(), max_images });
    cx.observe_release(&e, |cache, cx| {
      for (_, mut item) in std::mem::take(&mut cache.inner) {
        if let Some(Ok(image)) = item.get() {
          cx.drop_image(image, None);
        }
      }
    })
    .detach();
    e
  }
}

impl ImageCache for LruImageCache {
  fn load(
    &mut self, source: &Resource, window: &mut Window, cx: &mut App,
  ) -> Option<Result<Arc<RenderImage>, ImageCacheError>> {
    let hash = gpui::hash(source);

    // If already cached, move to back (most recently used)
    if self.inner.contains_key(&hash) {
      if let Some(pos) = self.order.iter().position(|&h| h == hash) {
        self.order.remove(pos);
        self.order.push_back(hash);
      }
      return self.inner.get_mut(&hash).and_then(|item: &mut ImageCacheItem| item.get());
    }

    // Evict oldest if at capacity
    while self.inner.len() >= self.max_images {
      if let Some(oldest) = self.order.pop_front()
        && let Some(mut item) = self.inner.swap_remove(&oldest)
        && let Some(Ok(image)) = item.get()
      {
        cx.drop_image(image, Some(window));
      }
    }

    // Load new image
    let fut = AssetLogger::<ImageAssetLoader>::load(source.clone(), cx);
    let task: Shared<Task<Result<Arc<RenderImage>, ImageCacheError>>> =
      cx.background_executor().spawn(fut).shared();
    self.inner.insert(hash, ImageCacheItem::Loading(task.clone()));
    self.order.push_back(hash);

    let entity = window.current_view();
    window
      .spawn(cx, {
        async move |cx| {
          _ = task.await;
          cx.on_next_frame(move |_, cx| {
            cx.notify(entity);
          });
        }
      })
      .detach();

    None
  }
}
