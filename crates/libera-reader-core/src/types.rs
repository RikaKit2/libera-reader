#[macro_export]
macro_rules! send_event {
  ($tx:expr, $event:expr) => {{
    if let Err(e) = $tx.send($event) {
      tracing::debug!(target: "event_tx", "Failed to send LibraryEvent: {e:?}");
    }
  }};
}

use gxhash::GxBuildHasher;
use indexmap::{IndexMap, IndexSet};

use crate::db::models::books::book::{Book, BookDir, BookPath};

pub type HashMap<K, V> = IndexMap<K, V, GxBuildHasher>;
pub type HashSet<T> = IndexSet<T, GxBuildHasher>;

pub const MUPDF_EXTENSIONS: [&str; 6] = ["pdf", "epub", "xps", "cbz", "mobi", "fb2"];

#[derive(Debug, Clone)]
pub enum LibraryEvent {
  BookAdded(Book),
  BooksBatchAdded(Vec<Book>),
  BookRemoved(BookPath),
  BookUpdated(Book),
  BookPathUpdated {
    old_path: BookPath,
    new_path: BookPath,
  },
  BookMarkAdded {
    book_path: BookPath,
  },
  BookMarkUpdated {
    book_path: BookPath,
  },
  BookMarkRemoved {
    book_path: BookPath,
  },
  DirRemoved(BookDir),
  /// Emitted when a thumbnail has been successfully extracted.
  /// UI should update the card to show the cover image.
  ThumbnailExtracted(BookPath),
}
