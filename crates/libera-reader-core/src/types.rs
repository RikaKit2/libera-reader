use crate::app_dirs::AppDirs;
use crate::db::models::Book;
use concurrent_queue::ConcurrentQueue;
use std::sync::Arc;
use crate::db::DataBase;

pub type HashSet<V> = gxhash::HashSet<V>;
pub type HashMap<K, V> = gxhash::HashMap<K, V>;
pub type HashMapExt = dyn gxhash::HashMapExt;


pub(crate) type BookPath = String;
pub(crate) type BookSize = u64;
pub(crate) type BookHash = String;
pub type DB = Arc<DataBase>;
#[allow(non_camel_case_types)]
pub type APP_DIRS = Arc<AppDirs>;
pub type NotCachedBooks = Arc<ConcurrentQueue<Box<Book>>>;
