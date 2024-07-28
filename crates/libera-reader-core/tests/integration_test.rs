use std::fs::File;
use std::path::PathBuf;
use std::time::Duration;

use libera_reader_core::app_core::AppCore;

#[derive(Debug)]
struct TestDirs {
  pub tests_files_dir: PathBuf,
  pub path_to_db: PathBuf,
}

impl TestDirs {
  pub fn new() -> Self {
    let proj_root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tests_dir = proj_root_dir.join("tests");
    let tests_files_dir = tests_dir.join("files");
    let path_to_db = tests_files_dir.join("test.db");
    Self {
      tests_files_dir,
      path_to_db,
    }
  }
}

struct NotifyServiceTest {
  dirs: TestDirs,
  pub app_core: AppCore,
  path_to_fist_book: PathBuf,
  path_to_second_book: PathBuf,
  path_to_fist_dir: PathBuf,
  path_to_second_dir: PathBuf,
}

impl NotifyServiceTest {
  pub async fn new() -> Self {
    let dirs = TestDirs::new();
    match std::fs::remove_dir_all(&dirs.tests_files_dir) {
      Ok(_) => {}
      Err(_) => {}
    }
    match std::fs::create_dir(&dirs.tests_files_dir) {
      Ok(_) => {}
      Err(_) => {}
    }
    let path_to_db = dirs.path_to_db.to_str().unwrap().to_string();
    let app_core = AppCore::new(Option::from(path_to_db)).await.unwrap();
    Self {
      path_to_fist_book: dirs.tests_files_dir.join("first_book").with_extension("pdf"),
      path_to_second_book: dirs.tests_files_dir.join("second_book").with_extension("pdf"),
      path_to_fist_dir: dirs.tests_files_dir.join("first_dir"),
      path_to_second_dir: dirs.tests_files_dir.join("second_dir"),
      dirs,
      app_core,
    }
  }
  pub fn create_first_book(&self) {
    File::create(&self.path_to_fist_book).unwrap();
  }
  pub fn rename_fist_book_to_second(&mut self) {
    std::fs::rename(&self.path_to_fist_book, &self.path_to_second_book).unwrap();
  }
  pub fn move_second_book_to_first_dir(&self) {
    std::fs::create_dir(&self.path_to_fist_dir).unwrap();
    let book_in_first_dir = self.dirs.tests_files_dir.join("first_dir")
      .join("second_book").with_extension("pdf");

    std::process::Command::new("mv").args(&[
      &self.path_to_second_book.to_str().unwrap(),
      &book_in_first_dir.to_str().unwrap()
    ]).spawn().unwrap();
    tracing::info!("\n move second_book to first_dir");
  }
  pub fn rename_first_dir_to_second(&self) {
    std::fs::rename(&self.path_to_fist_dir, &self.path_to_second_dir).unwrap();
  }
  pub fn drop_second_dir(&self) {
    std::process::Command::new("rm").args(&[
      "-r",
      &self.path_to_second_dir.to_str().unwrap(),
    ]).spawn().unwrap();
  }
  pub fn drop_files(&self) {
    std::fs::remove_dir_all(&self.dirs.tests_files_dir).unwrap();
    std::fs::create_dir(&self.dirs.tests_files_dir).unwrap();
  }
}

#[cfg(not(target_os = "windows"))]
#[tokio::test]
async fn test_notify_service() {
  let subscriber = tracing_subscriber::fmt()
    .pretty()
    .without_time()
    // Use a more compact, abbreviated log format
    .compact()
    // Display source code file paths
    .with_file(true)
    // Display source code line numbers
    .with_line_number(true)
    // Display the thread ID an event was recorded on
    .with_thread_ids(true)
    // Don't display the event's target (module path)
    .with_target(true)
    // Build the subscriber
    .finish();
  tracing::subscriber::set_global_default(subscriber).unwrap();
  let first_book = "first_book.pdf".to_string();
  let second_book = "second_book.pdf".to_string();
  let first_dir = "first_dir".to_string();
  let second_dir = "second_dir".to_string();

  let mut test = NotifyServiceTest::new().await;
  let path_to_scan = test.dirs.tests_files_dir.to_str().unwrap().to_string();

  test.app_core.settings_manager.set_path_to_scan(path_to_scan).await;
  test.app_core.run_notify_service().await;

  // file creation test
  test.create_first_book();
  tokio::time::sleep(Duration::from_millis(100)).await;

  let book = test.app_core.get_book_by_name(first_book.clone()).await.unwrap();
  assert_eq!(&first_book, &book.file_name);

  // file rename test
  test.rename_fist_book_to_second();
  tokio::time::sleep(Duration::from_millis(200)).await;

  let book = test.app_core.get_book_by_name(second_book.clone()).await.unwrap();
  assert_eq!(&second_book, &book.file_name);

  // // file movement test
  test.move_second_book_to_first_dir();
  tokio::time::sleep(Duration::from_millis(500)).await;

  let book = test.app_core.get_book_by_name(second_book.clone()).await.unwrap();
  assert_eq!(first_dir.clone(), book.dir_name);

  // dir renaming test
  test.rename_first_dir_to_second();
  tokio::time::sleep(Duration::from_millis(200)).await;
  //
  let book = test.app_core.get_book_by_name(second_book.clone()).await.unwrap();
  assert_eq!(second_dir.clone(), book.dir_name);

  // directory deletion test
  test.drop_second_dir();
  tokio::time::sleep(Duration::from_millis(200)).await;

  match test.app_core.get_book_by_name(second_book).await {
    None => {}
    Some(_) => {
      panic!();
    }
  }
  // drop tmp files
  test.drop_files();
}

