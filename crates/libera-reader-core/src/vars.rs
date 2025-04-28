use crate::app_dirs::AppDirs;
use crate::models::{Book, Settings, TargetExt};
use crate::types::NotifyEvents;
use concurrent_queue::ConcurrentQueue;
use notify::RecommendedWatcher;
use once_cell::sync::Lazy;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, RwLock};


pub(crate) static NOTIFY_EVENTS: Lazy<ConcurrentQueue<NotifyEvents>> = Lazy::new(|| ConcurrentQueue::unbounded());
pub(crate) static WATCHER: Lazy<Arc<Mutex<RecommendedWatcher>>> = Lazy::new(|| {
  Arc::from(Mutex::from(
    notify::recommended_watcher(move |res| NOTIFY_EVENTS.push(res).unwrap()).unwrap(),
  ))
});
pub(crate) static NOT_CACHED_BOOKS: Lazy<ConcurrentQueue<Box<Book>>> = Lazy::new(|| ConcurrentQueue::unbounded());

pub static APP_DIRS: Lazy<RwLock<AppDirs>> = Lazy::new(|| Default::default());
pub static SHUTDOWN: AtomicBool = AtomicBool::new(false);
pub static TARGET_EXT: Lazy<RwLock<TargetExt>> = Lazy::new(|| Default::default());
pub static SETTINGS: Lazy<RwLock<Settings>> = Lazy::new(|| Default::default());
