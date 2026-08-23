use anyhow::Result;
use std::hash::{BuildHasher, Hasher};
use std::path::Path;
use tracing::Level;

use crate::utils::logger::Formatter;
pub mod logger;
pub use logger::{debug, error, timing, title};

pub fn create_subscriber() -> Result<()> {
  let subscriber = tracing_subscriber::fmt()
    .event_format(Formatter { show_location: false })
    .with_max_level(Level::DEBUG)
    .finish();
  tracing::subscriber::set_global_default(subscriber)?;
  Ok(())
}

pub fn create_debug_subscriber() -> Result<()> {
  let subscriber = tracing_subscriber::fmt()
    .event_format(Formatter { show_location: true })
    .with_max_level(Level::DEBUG)
    .finish();
  tracing::subscriber::set_global_default(subscriber)?;
  Ok(())
}

pub async fn calc_file_hash<P: AsRef<Path>>(path_to_file: P) -> Result<String> {
  use tokio::io::AsyncReadExt;
  let mut hasher = gxhash::GxBuildHasher::with_seed(0).build_hasher();
  let mut file = tokio::fs::File::open(path_to_file).await?;
  let mut buffer = vec![0; 64 * 1024]; // Heap-allocated 64KB buffer prevents stack overflows
  loop {
    let bytes_read = file.read(&mut buffer).await?;
    if bytes_read == 0 {
      break;
    }
    hasher.write(&buffer[..bytes_read]);
  }
  Ok(data_encoding::HEXLOWER.encode(&hasher.finish().to_ne_bytes()))
}
