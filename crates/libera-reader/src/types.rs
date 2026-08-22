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

use crate::db::models::books::book::{BookDir, BookPath, BookSnapshot};

pub type HashMap<K, V> = IndexMap<K, V, GxBuildHasher>;
pub type HashSet<T> = IndexSet<T, GxBuildHasher>;

pub const MUPDF_EXTENSIONS: [&str; 6] = ["pdf", "epub", "xps", "cbz", "mobi", "fb2"];

#[derive(Debug, Clone)]
pub enum LibraryEvent {
  BookAdded(BookSnapshot),
  BooksBatchAdded(Vec<BookSnapshot>),
  BookRemoved(BookPath),
  BookUpdated(BookSnapshot),
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

// `LibraryEvent` carries UI-ready `BookSnapshot`s instead of full `Book`
// structs, so services never push more than the UI needs. Full `Book` is only
// used internally by the database and services that need `bookmarks`,
// `book_path`, or `book_size`.
