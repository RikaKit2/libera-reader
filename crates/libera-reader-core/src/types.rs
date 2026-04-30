use concurrent_queue::ConcurrentQueue;
use gxhash::GxBuildHasher;
use indexmap::{IndexMap, IndexSet};
use std::sync::Arc;

use crate::db::models::books::book::{Book, BookDir, BookPath};

pub type HashMap<K, V> = IndexMap<K, V, GxBuildHasher>;
pub type HashSet<T> = IndexSet<T, GxBuildHasher>;
pub type NotCachedBooks = Arc<ConcurrentQueue<Box<BookPath>>>;

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
