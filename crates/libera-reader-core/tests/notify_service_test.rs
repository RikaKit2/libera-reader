mod file_crud_lib;

use crate::file_crud_lib::*;
use tracing::Level;


#[cfg(not(target_os = "windows"))]
#[tokio::test(flavor = "multi_thread")]
async fn notify_service_test() {
  let subscriber = tracing_subscriber::fmt()
    .pretty()
    .without_time()
    .compact()
    .with_file(false)
    .with_line_number(true)
    .with_thread_ids(false)
    .with_target(false)
    .with_max_level(Level::DEBUG)
    .finish();
  tracing::subscriber::set_global_default(subscriber).unwrap();

  let mut fc_lib = FileCrudLib::new(TestMode::Notify, "tmp_notify");
  fc_lib.run_tests().await;
}
