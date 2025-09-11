use concurrent_queue::ConcurrentQueue;
use std::sync::Arc;

pub type HashSet<V> = gxhash::HashSet<V>;
pub type HashMap<K, V> = gxhash::HashMap<K, V>;
pub type HashMapExt = dyn gxhash::HashMapExt;

pub(crate) type BookPath = String;
pub(crate) type BookSize = u64;
pub(crate) type BookHash = String;
pub type NotCachedBooks = Arc<ConcurrentQueue<Box<BookType>>>;


#[derive(Debug, Clone)]
pub enum BookType {
  Unique(BookSize),
  Hashed(BookHash),
}