use crate::app_dirs::AppDirs;
use crate::db::models::{Book, Settings, TargetExt, Theme};
use concurrent_queue::ConcurrentQueue;
use native_db::Database;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[cfg(feature = "release")]
type HashBuilder = gxhash::GxBuildHasher;
#[cfg(feature = "release")]
pub type HashSet<V> = gxhash::HashSet<V>;
#[cfg(feature = "release")]
pub type HashMap<K, V> = gxhash::HashMap<K, V>;
#[cfg(feature = "release")]
pub type HashMapExt = gxhash::HashMapExt;


#[cfg(feature = "dev")]
pub type HashBuilder = std::hash::BuildHasherDefault<std::hash::DefaultHasher>;
#[cfg(feature = "dev")]
pub type HashSet<V> = std::collections::HashSet<V>;
#[cfg(feature = "dev")]
pub type HashMap<K, V> = std::collections::HashMap<K, V>;

#[cfg(not(any(feature = "release", feature = "dev")))]
compile_error!("Either `release` or `dev` feature must be enabled!");

pub(crate) type BookPath = String;
pub(crate) type BookSize = String;
pub(crate) type FileSize = String;
pub(crate) type BookHash = String;
pub(crate) type NotifyEvents = notify::Result<notify::Event>;
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MutoolErr {
  SIGSEGV,
  MutoolProcessFailed,
  OtherErr,
}

#[derive(Debug)]
pub enum Error {
  PathToDataDirIsNone,
  PathToScanIsNone,
}
pub type DB = Arc<Database<'static>>;
#[allow(non_camel_case_types)]
pub type TARGET_EXT = Arc<RwLock<TargetExt>>;
#[allow(non_camel_case_types)]
pub type APP_DIRS = Arc<RwLock<AppDirs>>;
pub type NotCachedBooks = Arc<ConcurrentQueue<Box<Book>>>;
#[allow(non_camel_case_types)]
pub type SETTINGS = Arc<RwLock<Settings>>;
#[allow(non_camel_case_types)]
pub type THEME = Arc<RwLock<Theme>>;
