use tokio::sync::mpsc;

use crate::db::models::books::book::BookPath;

/// Sender channel type for not-cached books queue.
pub type NotCachedBooksTx = mpsc::UnboundedSender<BookPath>;

/// Receiver channel type for not-cached books queue.
pub type NotCachedBooksRx = mpsc::UnboundedReceiver<BookPath>;

/// Lightweight, cloneable wrapper around the channel sender for notifying
/// the background thumbnail extraction service about new or updated books.
#[derive(Clone)]
pub struct NotCachedBooks {
  tx: NotCachedBooksTx,
}

impl gpui::Global for NotCachedBooks {}

impl NotCachedBooks {
  pub fn new(tx: NotCachedBooksTx) -> Self {
    Self { tx }
  }

  /// Create a new unbounded channel pair (sender wrapper + receiver).
  pub fn channel() -> (Self, NotCachedBooksRx) {
    let (tx, rx) = mpsc::unbounded_channel();
    (Self { tx }, rx)
  }

  #[inline(always)]
  pub fn tx(&self) -> &NotCachedBooksTx {
    &self.tx
  }

  #[inline(always)]
  pub fn send(&self, path: BookPath) -> Result<(), mpsc::error::SendError<BookPath>> {
    self.tx.send(path)
  }
}
