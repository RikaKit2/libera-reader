use crate::types::NotCachedBook;
use gxhash::HashSet;
use once_cell::sync::Lazy;
use std::collections::VecDeque;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, RwLock};
use crate::services::notify_service::watchdog::Watchdog;


pub(crate) static SHUTDOWN: AtomicBool = AtomicBool::new(false);
pub const DB_NAME: &str = "libera_reader.redb";
pub(crate) static TARGET_EXT: Lazy<Arc<RwLock<HashSet<String>>>> = Lazy::new(|| Default::default());
pub(crate) static WATCHDOG: Lazy<Watchdog> = Lazy::new(|| Watchdog::new());
pub(crate) static NOT_CACHED_BOOKS: Lazy<Arc<RwLock<VecDeque<NotCachedBook>>>> = Lazy::new(|| Default::default());
pub(crate) static PATH_TO_SCAN: Lazy<Arc<RwLock<Option<String>>>> = Lazy::new(|| Default::default());
