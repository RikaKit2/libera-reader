mod test_lib;

use crate::test_lib::*;
use anyhow::Result;

#[cfg(not(target_os = "windows"))]
#[tokio::test(flavor = "current_thread")]
async fn passive_scan_test() -> Result<()> {
  utils::create_subscriber()?;
  let mut test_lib = TestLib::new(TestMode::ScanService, "passive_scan").await?;
  test_lib.run().await?;
  Ok(())
}
