use crate::ui::components::books_grid::cache::{BoundedCache, CoverState};
use crate::ui::components::books_grid::image_utils::load_thumbnail;
use gpui::SharedString;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

/// Background cover loader.
///
/// Each request carries the grid position (`idx`, only used for the
/// in-view check), the **book id** (the cache key), and the PNG path.
pub fn spawn_background_loader(
  runtime: &Runtime, cache: Arc<Mutex<BoundedCache>>, visible_start: Arc<AtomicUsize>,
  visible_end: Arc<AtomicUsize>,
  mut load_rx: tokio::sync::mpsc::UnboundedReceiver<(usize, SharedString, PathBuf)>,
  notify_tx: tokio::sync::mpsc::UnboundedSender<()>,
) {
  let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).clamp(2, 6);
  let semaphore = Arc::new(tokio::sync::Semaphore::new(threads));

  runtime.spawn(async move {
    while let Some((idx, id, path)) = load_rx.recv().await {
      let permit = semaphore.clone().acquire_owned().await.unwrap();

      let s = visible_start.load(Ordering::Relaxed);
      let e = visible_end.load(Ordering::Relaxed);

      // Cancel stale requests for books that scrolled out of view.
      if idx < s || idx >= e {
        cache.lock().unwrap().pop_loading(&id);
        drop(permit);
        continue;
      }

      let cache_worker = cache.clone();
      let notify_worker = notify_tx.clone();

      tokio::task::spawn_blocking(move || {
        let state = match load_thumbnail(&path) {
          Some(img) => CoverState::Loaded(img),
          None => CoverState::Failed,
        };
        cache_worker.lock().unwrap().insert(id, state);
        let _ = notify_worker.send(());
        drop(permit);
      });
    }
  });
}
