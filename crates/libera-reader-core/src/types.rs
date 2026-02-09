use concurrent_queue::ConcurrentQueue;
use gxhash::GxBuildHasher;
use indexmap::{IndexMap, IndexSet};
use std::sync::Arc;

use crate::db::models::books::book::BookPath;

pub type HashMap<K, V> = IndexMap<K, V, GxBuildHasher>;
pub type HashSet<T> = IndexSet<T, GxBuildHasher>;
pub type NotCachedBooks = Arc<ConcurrentQueue<Box<BookPath>>>;
