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
  pub fn drop_files(&self) {
    std::fs::remove_dir_all(&self.dirs.tests_files_dir).unwrap();
    std::fs::create_dir(&self.dirs.tests_files_dir).unwrap();
  }
}

#[tokio::test]
async fn test_notify_service() {
  let mut test = NotifyServiceTest::new().await;
  let path_to_scan = test.dirs.tests_files_dir.to_str().unwrap().to_string();

  test.app_core.settings_manager.set_path_to_scan(path_to_scan).await;
  test.app_core.run_services().await;

  test.create_first_book();
  tokio::time::sleep(Duration::from_millis(100)).await;
  test.rename_fist_book_to_second();
  tokio::time::sleep(Duration::from_millis(100)).await;

  let path_to_book = test.path_to_second_book.to_str().unwrap().to_string();
  let book = test.app_core.get_book(path_to_book).await.unwrap();
  assert_eq!("second_book.pdf".to_string(), book.file_name);

  std::fs::create_dir(&test.path_to_fist_dir).unwrap();
  let new_path = test.dirs.tests_files_dir.join("first_dir")
    .join("second_book").with_extension("pdf");
  std::fs::rename(&test.path_to_second_book, new_path).unwrap();
  tokio::time::sleep(Duration::from_secs(1)).await;

  test.drop_files();
}

