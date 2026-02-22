mod test_lib;

use crate::test_lib::*;
use anyhow::Result;

#[tokio::test(flavor = "current_thread")]
async fn passive_scan_test() -> Result<()> {
  utils::create_subscriber()?;
  let mut test_lib = TestLib::new(TestMode::ScanService, "scan_service").await?;
  test_lib.run().await?;
  Ok(())
}
