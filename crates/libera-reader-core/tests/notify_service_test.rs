mod test_lib;

use crate::test_lib::*;
use anyhow::Result;

#[tokio::test(flavor = "multi_thread")]
async fn notify_service_test() -> Result<()> {
  utils::create_subscriber()?;
  let mut test_lib = TestLib::new(TestMode::Notify, "notify").await?;
  test_lib.run().await?;
  Ok(())
}
