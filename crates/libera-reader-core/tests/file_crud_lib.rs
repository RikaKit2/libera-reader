use libera_reader_core::core::Core;
use libera_reader_core::models::Book;
use libera_reader_core::vars::DB_NAME;
use std::fs::{create_dir, remove_dir_all, remove_file, rename, File};
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info};


#[allow(dead_code)]
pub enum TestMode {
  Notify,
  DirScan,
}

const TIME_BETWEEN_TESTS: u64 = 300;
const FIRST_BOOK: &str = "first_book.pdf";
pub const SECOND_BOOK: &str = "second_book.pdf";
const FIRST_DIR: &str = "first_dir";
const SECOND_DIR: &str = "second_dir";


pub struct FileCrudLib {
  first_book: PathBuf,
  second_book: PathBuf,

  fist_dir: PathBuf,
  second_dir: PathBuf,

  tmp_dir: PathBuf,
  proj_root_dir: PathBuf,
  core: Core,
  test_mode: TestMode,
}

impl FileCrudLib {
  pub fn new(test_mode: TestMode, tmp_dir_name: &str) -> Self {
    let proj_root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tmp_dir = proj_root_dir.join("tests").join(tmp_dir_name);
    Self {
      first_book: tmp_dir.join(&FIRST_BOOK),
      second_book: tmp_dir.join(&SECOND_BOOK),

      fist_dir: tmp_dir.join(&FIRST_DIR),
      second_dir: tmp_dir.join(&SECOND_DIR),

      tmp_dir,
      proj_root_dir,
      core: Core::new(),
      test_mode,
    }
  }
  pub async fn create_first_book(&mut self) {
    info!("Create first book");
    assert!(File::create(&self.first_book).is_ok());
    match self.test_mode {
      TestMode::Notify => { sleep(Duration::from_millis(TIME_BETWEEN_TESTS)).await; }
      TestMode::DirScan => { self.core.services.launch_dir_scan_service(true).await; }
    };
    self.test_fn(&self.first_book.to_string2(), |book: &Book| assert_eq!(&FIRST_BOOK, &book.book_name));
  }
  pub async fn rename_first_book_to_second(&mut self) {
    info!("File rename test: rename first book to second");
    match self.test_mode {
      TestMode::Notify => {
        let args = [&self.first_book.to_str().unwrap(), &self.second_book.to_str().unwrap()];
        assert!(Command::new("mv").args(args).spawn().is_ok());
        sleep(Duration::from_millis(TIME_BETWEEN_TESTS)).await;
      }
      TestMode::DirScan => {
        assert!(rename(&self.first_book, &self.second_book).is_ok());
        self.core.services.launch_dir_scan_service(true).await;
      }
    };
    self.test_fn(&self.second_book.to_string2(), |book: &Book| assert_eq!(&SECOND_BOOK, &book.book_name));
  }
  pub async fn move_second_book_to_first_dir(&mut self) {
    info!("File movement test: move second book to first dir");
    assert!(create_dir(&self.fist_dir).is_ok());
    let book_in_first_dir = self.tmp_dir.join(&FIRST_DIR).join(&SECOND_BOOK);
    match self.test_mode {
      TestMode::Notify => {
        let args = [&self.second_book.to_str().unwrap(), book_in_first_dir.parent().unwrap().to_str().unwrap()];
        assert!(Command::new("mv").args(args).spawn().is_ok());
        sleep(Duration::from_millis(TIME_BETWEEN_TESTS)).await;
      }
      TestMode::DirScan => {
        assert!(rename(&self.second_book, &book_in_first_dir).is_ok());
        self.core.services.launch_dir_scan_service(true).await;
      }
    }
    self.second_book = book_in_first_dir;
    self.test_fn(&self.second_book.to_string2(), |book: &Book| assert_eq!(&FIRST_DIR, &book.dir_name));
  }
  pub async fn rename_first_dir_to_second(&mut self) {
    info!("Dir renaming test: rename_first_dir_to_second");
    assert!(rename(&self.fist_dir, &self.second_dir).is_ok());
    match self.test_mode {
      TestMode::Notify => { sleep(Duration::from_millis(TIME_BETWEEN_TESTS)).await; }
      TestMode::DirScan => { self.core.services.launch_dir_scan_service(true).await; }
    }

    self.second_book = self.tmp_dir.join(&SECOND_DIR).join(&SECOND_BOOK);
    self.test_fn(&self.second_book.to_string2(), |book: &Book| assert_eq!(&SECOND_DIR, &book.dir_name));
  }
  pub async fn rename_second_book_to_first_in_second_dir(&mut self) {
    info!("File rename test2: rename second book to first in second dir");

    self.first_book = self.tmp_dir.join(&SECOND_DIR).join(&FIRST_BOOK);
    assert!(rename(&self.second_book, &self.first_book).is_ok());
    match self.test_mode {
      TestMode::Notify => { sleep(Duration::from_millis(TIME_BETWEEN_TESTS)).await; }
      TestMode::DirScan => { self.core.services.launch_dir_scan_service(true).await; }
    }

    self.test_fn(&self.first_book.to_string2(), |book: &Book| assert_eq!(&FIRST_BOOK, &book.book_name));
  }
  pub async fn drop_second_dir(&mut self) {
    info!("Dir deletion test: drop_second_dir");
    assert!(remove_dir_all(&self.second_dir).is_ok());
    match self.test_mode {
      TestMode::Notify => { sleep(Duration::from_millis(TIME_BETWEEN_TESTS)).await; }
      TestMode::DirScan => { self.core.services.launch_dir_scan_service(true).await; }
    }
    match self.core.book_api.get_book_by_path(&self.first_book.to_string2()) {
      Ok(poss_book) => {
        assert_eq!(poss_book, None, "there shouldn't be a book");
      }
      Err(_) => {}
    };
  }
  pub fn drop_files(&self) {
    match remove_dir_all(&self.tmp_dir) {
      Ok(_) => {}
      Err(e) => error!("error when deleting tests_files_dir: {:?}", e),
    };
    match remove_file(&self.proj_root_dir.join(DB_NAME)) {
      Ok(_) => {}
      Err(_) => {}
    };
    match create_dir(&self.tmp_dir) {
      Ok(_) => {}
      Err(e) => error!("error when creating tests_files_dir: {:?}", e),
    };
  }
  fn test_fn<F>(&self, book_path_in_db: &String, assert_fn: F)
  where
    F: Fn(&Book),
  {
    match self.core.book_api.get_book_by_path(book_path_in_db) {
      Ok(res) => {
        match res {
          None => {
            panic!("book in db not found: {:?}", book_path_in_db)
          }
          Some(book) => {
            assert_fn(&book);
          }
        }
      }
      Err(e) => {
        panic!("{:?}", e)
      }
    }
  }
  pub async fn run_tests(&mut self) {
    self.drop_files();
    self.core.settings.set_path_to_scan(self.tmp_dir.to_str().unwrap().to_string());

    match self.test_mode {
      TestMode::Notify => {
        self.core.services.run_notify();
      }
      TestMode::DirScan => {}
    }
    self.create_first_book().await;
    self.rename_first_book_to_second().await;
    self.move_second_book_to_first_dir().await;
    self.rename_first_dir_to_second().await;
    self.rename_second_book_to_first_in_second_dir().await;
    self.drop_second_dir().await;
  }
}
pub trait EasyString {
  fn to_string2(&self) -> String;
}

impl EasyString for PathBuf {
  fn to_string2(&self) -> String {
    self.to_str().unwrap().to_string()
  }
}
impl Drop for FileCrudLib {
  fn drop(&mut self) {
    self.core.services.stop_all_services();
    self.drop_files();
  }
}
