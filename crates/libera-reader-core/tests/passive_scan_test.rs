mod file_crud_lib;

use crate::file_crud_lib::*;
use anyhow::Result;
use libera_reader_core::db::get_models;
use native_db::Models;
use once_cell::sync::Lazy;

#[cfg(not(target_os = "windows"))]
#[test]
fn passive_scan_test() -> Result<()> {
  utils::create_subscriber()?;
  pub static MODELS: Lazy<Models> = Lazy::new(|| get_models().unwrap());
  let mut fc_lib = FileCrudLib::new(TestMode::PassiveScan, "tmp_dir_scan", &MODELS)?;
  fc_lib.run_tests()?;
  Ok(())
}
