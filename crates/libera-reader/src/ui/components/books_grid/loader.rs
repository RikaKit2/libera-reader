use crate::ui::components::books_grid::cache::{BoundedCache, CoverState};
use crate::ui::components::books_grid::image_utils::load_thumbnail;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

pub fn spawn_background_loader(
  runtime: &Runtime, cache: Arc<Mutex<BoundedCache>>, visible_start: Arc<AtomicUsize>,
  visible_end: Arc<AtomicUsize>,
  mut load_rx: tokio::sync::mpsc::UnboundedReceiver<(usize, PathBuf)>,
  notify_tx: tokio::sync::mpsc::UnboundedSender<()>,
) {
  let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).clamp(2, 6);
  let semaphore = Arc::new(tokio::sync::Semaphore::new(threads));

  runtime.spawn(async move {
    while let Some((idx, path)) = load_rx.recv().await {
      let permit = semaphore.clone().acquire_owned().await.unwrap();

      let s = visible_start.load(Ordering::Relaxed);
      let e = visible_end.load(Ordering::Relaxed);

      if idx < s || idx >= e {
        let mut c = cache.lock().unwrap();
        if let Some(CoverState::Loading) = c.inner.peek(&idx) {
          c.inner.pop(&idx);
        }
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
        cache_worker.lock().unwrap().insert(idx, state);
        let _ = notify_worker.send(());
        drop(permit);
      });
    }
  });
}
