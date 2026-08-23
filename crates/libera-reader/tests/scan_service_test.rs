#[path = "mod.rs"]
mod test_lib;

use anyhow::Result;
use test_lib::*;

#[tokio::test(flavor = "current_thread")]
async fn passive_scan_test() -> Result<()> {
  libera_reader::utils::create_subscriber()?;
  let mut test_lib = TestLib::new(TestMode::ScanService, "scan_service").await?;
  test_lib.run().await?;
  Ok(())
}
