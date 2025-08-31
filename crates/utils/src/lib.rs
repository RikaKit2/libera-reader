use anyhow::Result;
use std::hash::{BuildHasher, Hasher};
use std::path::{Path, PathBuf};
use tracing::Level;

pub async fn get_file_size(path_to_file: &PathBuf) -> Result<u64> {
  let metadata = tokio::fs::metadata(path_to_file).await?;
  let file_size = metadata.len();
  Ok(file_size)
}
#[rustfmt::skip]
pub fn create_subscriber() ->Result<()> {
  let subscriber = tracing_subscriber::fmt()
    .pretty()
    .without_time()
    .compact()
    .with_file(false)
    .with_line_number(false)
    .with_thread_ids(false)
    .with_target(false)
    .with_max_level(Level::INFO)
    .finish();
  tracing::subscriber::set_global_default(subscriber)?;
  Ok(())
}
#[rustfmt::skip]
pub fn create_debug_subscriber() ->Result<()> {
  let subscriber = tracing_subscriber::fmt()
    .pretty()
    .without_time()
    .compact()
    .with_file(true)
    .with_line_number(true)
    .with_thread_ids(false)
    .with_target(false)
    .with_max_level(Level::DEBUG)
    .finish();
  tracing::subscriber::set_global_default(subscriber)?;
  Ok(())
}
pub async fn calc_file_hash<P: AsRef<Path>>(path_to_file: P) -> Result<String> {
  use tokio::io::AsyncReadExt;
  let mut hasher = gxhash::GxBuildHasher::with_seed(0).build_hasher();
  let mut file = tokio::fs::File::open(path_to_file).await?;
  loop {
    let mut buffer = [0; 1024 * 1024];
    let bytes_read = file.read(&mut buffer).await?;
    if bytes_read == 0 {
      break;
    }
    hasher.write(&buffer[..bytes_read]);
  }
  Ok(data_encoding::HEXLOWER.encode(&hasher.finish().to_ne_bytes()))
}
