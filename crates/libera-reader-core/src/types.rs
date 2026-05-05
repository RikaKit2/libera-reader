use gxhash::GxBuildHasher;
use indexmap::{IndexMap, IndexSet};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;

use crate::db::models::books::book::{Book, BookDir, BookPath};

pub type HashMap<K, V> = IndexMap<K, V, GxBuildHasher>;
pub type HashSet<T> = IndexSet<T, GxBuildHasher>;

// New struct for smart management of the thumbnail extraction queue
pub struct ExtractorQueues {
  pub high_tx: mpsc::UnboundedSender<BookPath>,
  pub low_tx: mpsc::UnboundedSender<BookPath>,
  pub processing_now: Arc<RwLock<HashSet<BookPath>>>,
}

impl ExtractorQueues {
  pub fn new() -> (Self, mpsc::UnboundedReceiver<BookPath>, mpsc::UnboundedReceiver<BookPath>) {
    let (high_tx, high_rx) = mpsc::unbounded_channel();
    let (low_tx, low_rx) = mpsc::unbounded_channel();
    let processing_now = Arc::new(RwLock::new(HashSet::default()));

    (Self { high_tx, low_tx, processing_now }, high_rx, low_rx)
  }
}

impl Clone for ExtractorQueues {
  fn clone(&self) -> Self {
    Self {
      high_tx: self.high_tx.clone(),
      low_tx: self.low_tx.clone(),
      processing_now: Arc::clone(&self.processing_now),
    }
  }
}

pub const MUPDF_EXTENSIONS: [&str; 6] = ["pdf", "epub", "xps", "cbz", "mobi", "fb2"];

#[derive(Debug, Clone)]
pub enum LibraryEvent {
  BookAdded(Book),
  BookRemoved(BookPath),
  BookUpdated(Book),
  BookPathUpdated { old_path: BookPath, new_path: BookPath },
  BookMarkAdded { book_path: BookPath },
  BookMarkUpdated { book_path: BookPath },
  BookMarkRemoved { book_path: BookPath },
  DirRemoved(BookDir),
}
