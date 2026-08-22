use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use crate::db::models::books::book::BookPath;

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

  pub fn take_rx(&self) -> Option<mpsc::UnboundedReceiver<BookPath>> {
    self.rx.lock().unwrap().take()
  }

  pub fn tx(&self) -> &mpsc::UnboundedSender<BookPath> {
    &self.tx
  }
}
