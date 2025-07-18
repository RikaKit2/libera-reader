use crate::app_dirs::AppDirs;
use crate::db::models::{Book, TargetExt};
use concurrent_queue::ConcurrentQueue;
use native_db::Database;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

pub type HashSet<V> = gxhash::HashSet<V>;
pub type HashMap<K, V> = gxhash::HashMap<K, V>;
pub type HashMapExt = dyn gxhash::HashMapExt;


pub(crate) type BookPath = String;
pub(crate) type BookSize = String;
pub(crate) type BookHash = String;
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
