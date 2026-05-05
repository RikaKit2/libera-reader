use std::sync::{Arc, Mutex, RwLock};
use tokio::sync::mpsc;

use crate::db::models::books::book::BookPath;
use crate::types::{ExtractorQueues, HashSet};

#[derive(Clone)]
pub struct NotCachedBooks {
  inner: Arc<ExtractorQueues>,
  // Receivers are wrapped in Option and Mutex since they can only be taken once
  high_rx: Arc<Mutex<Option<mpsc::UnboundedReceiver<BookPath>>>>,
  low_rx: Arc<Mutex<Option<mpsc::UnboundedReceiver<BookPath>>>>,
}

impl NotCachedBooks {
  pub(crate) fn new() -> Self {
    let (queues, high_rx, low_rx) = ExtractorQueues::new();
    Self {
      inner: Arc::new(queues),
      high_rx: Arc::new(Mutex::new(Some(high_rx))),
      low_rx: Arc::new(Mutex::new(Some(low_rx))),
    }
  }

  pub fn inner(&self) -> &Arc<ExtractorQueues> {
    &self.inner
  }

  pub fn take_high_rx(&self) -> Option<mpsc::UnboundedReceiver<BookPath>> {
    self.high_rx.lock().ok().and_then(|mut opt| opt.take())
  }

  pub fn take_low_rx(&self) -> Option<mpsc::UnboundedReceiver<BookPath>> {
    self.low_rx.lock().ok().and_then(|mut opt| opt.take())
  }

  pub fn high_tx(&self) -> &mpsc::UnboundedSender<BookPath> {
    &self.inner.high_tx
  }

  pub fn low_tx(&self) -> &mpsc::UnboundedSender<BookPath> {
    &self.inner.low_tx
  }

  pub fn processing_now(&self) -> &Arc<RwLock<HashSet<BookPath>>> {
    &self.inner.processing_now
  }
}
