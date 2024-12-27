use crate::app_dirs::AppDirs;
use crate::book_api::BookApi;
use crate::services::Services;
use crate::settings::Settings;
use crate::types::NotCachedBook;
use once_cell::sync::Lazy;
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

pub static SETTINGS: Lazy<Arc<RwLock<Settings>>> = Lazy::new(|| Default::default());

pub static SERVICES: Lazy<Arc<RwLock<Services>>> = Lazy::new(|| Default::default());
pub static BOOK_API: Lazy<Arc<RwLock<BookApi>>> = Lazy::new(|| Default::default());
pub static APP_DIRS: Lazy<Result<AppDirs, Vec<String>>> = Lazy::new(|| AppDirs::new());
pub(crate) static NOT_CACHED_BOOKS: Lazy<Arc<RwLock<VecDeque<NotCachedBook>>>> = Lazy::new(|| Default::default());
