use concurrent_queue::ConcurrentQueue;
use std::sync::Arc;

use crate::db::models::books::book::BookPath;

#[derive(Clone)]
pub struct NotCachedBooks(pub Arc<ConcurrentQueue<Box<BookPath>>>);
impl NotCachedBooks {
  pub(crate) fn new() -> Self {
    Self(Arc::new(ConcurrentQueue::unbounded()))
  }
}
