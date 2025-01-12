use crate::app_dirs::AppDirs;
use crate::models::TargetExt;
use crate::types::NotifyEvents;
use crate::utils::NotCachedBook;
use concurrent_queue::ConcurrentQueue;
use notify::RecommendedWatcher;
use once_cell::sync::Lazy;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, RwLock};


pub const DB_NAME: &str = "libera_reader.redb";
pub(crate) static SHUTDOWN: AtomicBool = AtomicBool::new(false);
pub(crate) static NOTIFY_EVENTS: Lazy<ConcurrentQueue<NotifyEvents>> = Lazy::new(|| ConcurrentQueue::unbounded());
pub(crate) static WATCHER: Lazy<Arc<Mutex<RecommendedWatcher>>> = Lazy::new(||
  Arc::from(Mutex::from(notify::recommended_watcher(move |res| NOTIFY_EVENTS.push(res).unwrap()).unwrap()))
);
pub(crate) static NOT_CACHED_BOOKS: Lazy<ConcurrentQueue<NotCachedBook>> = Lazy::new(|| ConcurrentQueue::unbounded());

pub(crate) static PATH_TO_SCAN: Lazy<Arc<RwLock<Option<String>>>> = Lazy::new(|| Default::default());
pub static APP_DIRS: Lazy<Arc<RwLock<AppDirs>>> = Lazy::new(||
  Arc::from(RwLock::from(AppDirs::new().unwrap()))
);
pub(crate) static TARGET_EXT: Lazy<Arc<RwLock<TargetExt>>> = Lazy::new(|| Default::default());
