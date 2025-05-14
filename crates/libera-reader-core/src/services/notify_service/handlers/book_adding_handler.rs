use crate::db::crud;
use crate::db::models::TargetExt;
use crate::utils::calc_file_size_in_mb;
use measure_time_macro::measure_time;
use native_db::Database;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tracing::debug;

#[measure_time]
pub(crate) fn book_adding_handler(book_pathbuf: &PathBuf, db: &Database, target_ext: &Arc<RwLock<TargetExt>>) -> Duration {
  let start_time = std::time::Instant::now();
  let ext = book_pathbuf.extension().unwrap().to_str().unwrap().to_string();
  if target_ext.read().unwrap().contains(&ext) {
    let book_size = calc_file_size_in_mb(book_pathbuf);
    crud::book::add_book(book_pathbuf, book_size, db);
  }
  let total_time = start_time.elapsed();
  debug!("Function book_adding_handler executed in: {:?}", &total_time);
  total_time
}
