use libera_reader_core::core::Core;
use libera_reader_core::models::Book;
use libera_reader_core::vars::DB_NAME;
use std::fs::{create_dir, remove_dir_all, remove_file, rename, File};
use std::path::PathBuf;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;
use tracing::{error, info};


const TIME_BETWEEN_TESTS: u64 = 170;
const FIRST_BOOK: &str = "first_book.pdf";
pub const SECOND_BOOK: &str = "second_book.pdf";
const FIRST_DIR: &str = "first_dir";
const SECOND_DIR: &str = "second_dir";


pub(crate) struct NotifyTest {
  first_book: PathBuf,
  second_book: PathBuf,

  fist_dir: PathBuf,
  second_dir: PathBuf,

  pub(crate) tmp_dir: PathBuf,
  proj_root_dir: PathBuf,
  pub core: Core,
}

impl NotifyTest {
  pub(crate) fn new() -> Self {
    let proj_root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tmp_dir = proj_root_dir.join("tests").join("tmp_files");
    Self {
      first_book: tmp_dir.join(&FIRST_BOOK),
      second_book: tmp_dir.join(&SECOND_BOOK),

      fist_dir: tmp_dir.join(&FIRST_DIR),
      second_dir: tmp_dir.join(&SECOND_DIR),

      tmp_dir,
      proj_root_dir,
      core: Core::new().unwrap(),
    }
  }
  pub(crate) fn file_creation_test(&self) {
    assert!(File::create(&self.first_book).is_ok());
    sleep(Duration::from_millis(TIME_BETWEEN_TESTS));
    self.test_fn(&self.first_book.to_string2(), |book: &Book| assert_eq!(&FIRST_BOOK, &book.book_name));
    info!("File creation test: create first book IS PASSED");
  }
  pub(crate) fn file_rename_test(&self) {
    assert!(rename(&self.first_book, &self.second_book).is_ok());
    sleep(Duration::from_millis(TIME_BETWEEN_TESTS));
    self.test_fn(&self.second_book.to_string2(), |book: &Book| assert_eq!(&SECOND_BOOK, &book.book_name));
    info!("File rename test: rename first book to second IS PASSED");
  }
  pub(crate) fn file_movement_test(&mut self) {
    assert!(create_dir(&self.fist_dir).is_ok());
    let book_in_first_dir = self.tmp_dir.join(&FIRST_DIR).join(&SECOND_BOOK);

    let args = [&self.second_book.to_str().unwrap(), book_in_first_dir.parent().unwrap().to_str().unwrap()];
    assert!(Command::new("mv").args(args).spawn().is_ok());
    self.second_book = book_in_first_dir;

    sleep(Duration::from_millis(TIME_BETWEEN_TESTS));
    self.test_fn(&self.second_book.to_string2(), |book: &Book| assert_eq!(&FIRST_DIR, &book.dir_name));
    info!("File movement test: move second book to first dir IS PASSED");
  }
  pub(crate) fn dir_renaming_test(&mut self) {
    assert!(rename(&self.fist_dir, &self.second_dir).is_ok());
    sleep(Duration::from_millis(TIME_BETWEEN_TESTS));
    self.second_book = self.tmp_dir.join(&SECOND_DIR).join(&SECOND_BOOK);

    self.test_fn(&self.second_book.to_string2(), |book: &Book| assert_eq!(&SECOND_DIR, &book.dir_name));

    info!("Dir renaming test: rename_first_dir_to_second IS PASSED");
  }
  pub(crate) fn test_for_renaming_book_in_renamed_dir(&mut self) {
    self.first_book = self.tmp_dir.join(&SECOND_DIR).join(&FIRST_BOOK);
    let args = [self.second_book.to_string2(), self.first_book.to_string2()];
    assert!(Command::new("mv").args(args).spawn().is_ok());

    sleep(Duration::from_millis(TIME_BETWEEN_TESTS));
    self.test_fn(&self.first_book.to_string2(), |book: &Book| assert_eq!(&FIRST_BOOK, &book.book_name));

    info!("File rename test2: rename second book to first in second dir IS PASSED");
  }
  pub(crate) fn dir_deletion_test(&self) {
    assert!(remove_dir_all(&self.second_dir).is_ok());
    sleep(Duration::from_millis(TIME_BETWEEN_TESTS));
    assert!(self.core.book_api.get_book_by_path(&self.first_book.to_string2()).unwrap().is_none());
    info!("Dir deletion test: drop_second_dir IS PASSED");
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
}
pub(crate) trait EasyString {
  fn to_string2(&self) -> String;
}

impl EasyString for PathBuf {
  fn to_string2(&self) -> String {
    self.to_str().unwrap().to_string()
  }
}
impl Drop for NotifyTest {
  fn drop(&mut self) {
    self.drop_files();
  }
}
