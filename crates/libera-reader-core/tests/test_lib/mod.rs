use std::fs::File;
use std::path::PathBuf;

pub(crate) const FIRST_BOOK: &str = "first_book.pdf";
pub(crate) const SECOND_BOOK: &str = "second_book.pdf";
pub(crate) const FIRST_DIR: &str = "first_dir";
pub(crate) const SECOND_DIR: &str = "second_dir";

#[derive(Debug)]
pub(crate) struct TestDirs {
  pub tests_files_dir: PathBuf,
}

impl TestDirs {
  pub fn new() -> Self {
    let proj_root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tests_dir = proj_root_dir.join("tests");
    let tests_files_dir = tests_dir.join("files");
    Self {
      tests_files_dir,
    }
  }
}

pub(crate) struct NotifyServiceTest {
  pub(crate) dirs: TestDirs,
  pub(crate) path_to_fist_book: PathBuf,
  pub(crate) path_to_second_book: PathBuf,
  pub(crate) path_to_fist_dir: PathBuf,
  pub(crate) path_to_second_dir: PathBuf,
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
    Self {
      path_to_fist_book: dirs.tests_files_dir.join(&FIRST_BOOK).with_extension("pdf"),
      path_to_second_book: dirs.tests_files_dir.join(&SECOND_BOOK).with_extension("pdf"),
      path_to_fist_dir: dirs.tests_files_dir.join(&FIRST_DIR),
      path_to_second_dir: dirs.tests_files_dir.join(&SECOND_DIR),
      dirs,
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
    let book_in_first_dir = self.dirs.tests_files_dir.join(&FIRST_DIR)
      .join(&SECOND_BOOK).with_extension("pdf");

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

