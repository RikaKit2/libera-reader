use std::path::PathBuf;

use crate::db::DB;
use crate::db::models::book;
use anyhow::Result;
use tracing::info;

pub(crate) async fn book_deletion_handler(book_path: &PathBuf, db: &DB) -> Result<()> {
  let start_time = std::time::Instant::now();
  book::remove(book_path, db).await?;
  let total_time = start_time.elapsed();
  info!("Function book_deletion_handler executed in: {:?}", &total_time);
  Ok(())
}
