mod test_lib;
use crate::test_lib::*;
use std::time::Duration;


#[allow(non_snake_case)]
#[cfg(not(target_os = "windows"))]
#[tokio::test]
async fn test_notify_service() {
  let subscriber = tracing_subscriber::fmt()
    .pretty()
    .without_time()
    .compact()
    .with_file(true)
    .with_line_number(true)
    .finish();
  tracing::subscriber::set_global_default(subscriber).unwrap();
  let mut test = NotifyServiceTest::new().await;
  let path_to_scan = test.dirs.tests_files_dir.to_str().unwrap().to_string();

  test.app_core.settings.set_path_to_scan(path_to_scan).await;
  test.app_core.services.run_notify().await;

  let FIRST_BOOK_PATH = test.path_to_fist_book.to_str().unwrap().to_string();
  let SECOND_BOOK_PATH = test.path_to_second_book.to_str().unwrap().to_string();
  
  // file creation test
  test.create_first_book();
  tokio::time::sleep(Duration::from_millis(300)).await;

  let book = test.app_core.book_api.get_book_by_path(&FIRST_BOOK_PATH).unwrap();
  assert_eq!(&FIRST_BOOK, &book.book_name);

  // file rename test
  test.rename_fist_book_to_second();
  tokio::time::sleep(Duration::from_millis(300)).await;

  let book = test.app_core.book_api.get_book_by_path(&SECOND_BOOK_PATH).unwrap();
  assert_eq!(&SECOND_BOOK, &book.book_name);

  // // file movement test
  test.move_second_book_to_first_dir();
  tokio::time::sleep(Duration::from_millis(500)).await;

  let book = test.app_core.book_api.get_book_by_path(&SECOND_BOOK_PATH).unwrap();
  assert_eq!(&FIRST_DIR, &book.dir_name);

  // dir renaming test
  test.rename_first_dir_to_second();
  tokio::time::sleep(Duration::from_millis(300)).await;
  //
  let book = test.app_core.book_api.get_book_by_path(&SECOND_BOOK_PATH).unwrap();
  assert_eq!(&SECOND_DIR, &book.dir_name);

  // directory deletion test
  test.drop_second_dir();
  tokio::time::sleep(Duration::from_millis(300)).await;

  match test.app_core.book_api.get_book_by_path(&FIRST_BOOK_PATH) {
    None => {}
    Some(_) => {
      panic!();
    }
  }
  // drop tmp files
  test.drop_files();
}

