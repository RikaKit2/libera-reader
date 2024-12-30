use crate::types::{NotCachedBook, NotifyEvents};
use crossbeam_queue::SegQueue;
use gxhash::HashSet;
use notify::RecommendedWatcher;
use once_cell::sync::Lazy;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, RwLock};

pub const DB_NAME: &str = "libera_reader.redb";
pub(crate) static SHUTDOWN: AtomicBool = AtomicBool::new(false);
pub(crate) static TARGET_EXT: Lazy<Arc<RwLock<HashSet<String>>>> = Lazy::new(|| Default::default());
pub(crate) static NOTIFY_EVENTS: Lazy<SegQueue<NotifyEvents>> = Lazy::new(|| Default::default());
pub(crate) static WATCHER: Lazy<Arc<Mutex<RecommendedWatcher>>> = Lazy::new(||
  Arc::from(Mutex::from(notify::recommended_watcher(move |res| NOTIFY_EVENTS.push(res)).unwrap()))
);
pub(crate) static NOT_CACHED_BOOKS: Lazy<SegQueue<NotCachedBook>> = Lazy::new(|| Default::default());
pub(crate) static PATH_TO_SCAN: Lazy<Arc<RwLock<Option<String>>>> = Lazy::new(|| Default::default());
