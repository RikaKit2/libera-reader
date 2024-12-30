mod test_lib;

use crate::test_lib::*;
use std::panic::catch_unwind;
use tokio::runtime::Runtime;
use tracing::{info, Level};


#[allow(non_snake_case)]
#[test]
fn test_notify_service() {
  let subscriber = tracing_subscriber::fmt()
    .pretty()
    .without_time()
    .compact()
    .with_file(false)
    .with_line_number(true)
    .with_thread_ids(false)
    .with_target(false)
    .with_max_level(Level::INFO)
    .finish();
  tracing::subscriber::set_global_default(subscriber).unwrap();

  let result = catch_unwind(|| {
    Runtime::new().unwrap().block_on(async {
      let mut test = NotifyTest::new();
      test.drop_files();

      test.core.settings.set_path_to_scan(test.tmp_dir.to_str().unwrap().to_string());
      test.core.services.run_notify();

      test.file_creation_test().await;

      test.file_rename_test().await;

      test.file_movement_test().await;
      
      test.dir_renaming_test().await;
      
      test.test_for_renaming_book_in_renamed_dir().await;

      test.dir_deletion_test().await;
    })
  });

  match result {
    Ok(_) => info!("Notify service test completed successfully"),
    Err(e) => panic!("{:?}", e),
  }
}
