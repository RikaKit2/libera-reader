mod test_lib;

use crate::test_lib::*;
use anyhow::Result;
use libera_reader_core::db::get_models;
use native_db::Models;
use once_cell::sync::Lazy;

#[cfg(not(target_os = "windows"))]
#[tokio::test(flavor = "current_thread")]
async fn passive_scan_test() -> Result<()> {
  utils::create_subscriber()?;
  pub static MODELS: Lazy<Models> = Lazy::new(|| get_models().unwrap());
  let mut test_lib = TestLib::new(TestMode::PassiveScan, "passive_scan", &MODELS).await?;
  test_lib.run().await?;
  Ok(())
}
