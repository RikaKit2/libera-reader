mod test_lib;

use crate::test_lib::*;
use anyhow::Result;

#[cfg(not(target_os = "windows"))]
#[tokio::test(flavor = "current_thread")]
async fn notify_service_test() -> Result<()> {
  utils::create_subscriber()?;
  let mut test_lib = TestLib::new(TestMode::Notify, "notify").await?;
  test_lib.run().await?;
  Ok(())
}
