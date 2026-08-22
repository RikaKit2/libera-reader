use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use crate::db::models::books::book::BookPath;

/// Simple channel-based queue for thumbnail extraction.
/// Books are processed sequentially in FIFO order.
#[derive(Clone)]
pub struct NotCachedBooks {
  tx: mpsc::UnboundedSender<BookPath>,
  rx: Arc<Mutex<Option<mpsc::UnboundedReceiver<BookPath>>>>,
}

impl gpui::Global for NotCachedBooks {}

impl Default for NotCachedBooks {
  fn default() -> Self {
    Self::new()
  }
}

impl NotCachedBooks {
  pub fn new() -> Self {
    let (tx, rx) = mpsc::unbounded_channel();
    Self { tx, rx: Arc::new(Mutex::new(Some(rx))) }
  }

  /// Take the receiver (can only be called once).
  pub fn take_rx(&self) -> Option<mpsc::UnboundedReceiver<BookPath>> {
    self.rx.lock().unwrap().take()
  }

  /// Get a sender to push books into the extraction queue.
  pub fn tx(&self) -> &mpsc::UnboundedSender<BookPath> {
    &self.tx
  }
}
