use gxhash::GxBuildHasher;
use indexmap::{IndexMap, IndexSet};

pub type HashMap<K, V> = IndexMap<K, V, GxBuildHasher>;
pub type HashSet<T> = IndexSet<T, GxBuildHasher>;

pub const MUPDF_EXTENSIONS: [&str; 6] = ["pdf", "epub", "xps", "cbz", "mobi", "fb2"];
