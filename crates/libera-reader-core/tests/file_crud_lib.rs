use concurrent_queue::ConcurrentQueue;
use libera_reader_core::app_dirs::AppDirs;
use libera_reader_core::db::models::{Book, Settings, TargetExt};
use libera_reader_core::db::create_db_on_disk;
use libera_reader_core::services::Services;
use libera_reader_core::types::{NotCachedBooks, DB};
use std::fs::{create_dir, remove_dir_all, rename, File};
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, RwLock};
use std::thread::sleep;
use std::time::Duration;
use tracing::{debug, error, info};

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
  test_mode: TestMode,

  first_book: PathBuf,
  second_book: PathBuf,

  fist_dir: PathBuf,
  second_dir: PathBuf,
  tmp_dir: PathBuf,
  db: DB,
  services: Services,
}
impl FileCrudLib {
  pub fn new(test_mode: TestMode, tmp_dir_name: &str) -> Self {
    let proj_root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tmp_dir = proj_root_dir.join("test_files").join(tmp_dir_name);
    Self::drop_files(&tmp_dir);
    let app_dirs = AppDirs::new(tmp_dir.clone()).unwrap();
    let db = Arc::new(create_db_on_disk(app_dirs.inn.path_to_db.clone()).unwrap());
    Settings::create_if_not_exist(&db);
    let app_dirs = Arc::new(RwLock::new(app_dirs));
    let target_ext = Arc::new(RwLock::new(TargetExt::new(&db)));
    let not_cached_books = NotCachedBooks::new(ConcurrentQueue::unbounded());
    let services = Services::new(target_ext, app_dirs, db.clone(), not_cached_books);
    Self {
      first_book: tmp_dir.join(&FIRST_BOOK),
      second_book: tmp_dir.join(&SECOND_BOOK),
      fist_dir: tmp_dir.join(&FIRST_DIR),
      second_dir: tmp_dir.join(&SECOND_DIR),
      tmp_dir,
      test_mode,
      db,
      services,
    }
  }
  pub fn create_first_book(&mut self) {
    info!("Create first book");
    assert!(File::create(&self.first_book).is_ok());
    match self.test_mode {
      TestMode::Notify => { sleep(Duration::from_millis(TIME_BETWEEN_TESTS)); }
      TestMode::DirScan => { self.services.run_dir_scan(&self.tmp_dir.clone().to_string2()); }
    };
    self.test_fn(&self.first_book.to_string2(), |book: &Book| assert_eq!(&FIRST_BOOK, &book.book_name));
  }
  pub fn rename_first_book_to_second(&mut self) {
    info!("File rename test: rename first book to second");
    match self.test_mode {
      TestMode::Notify => {
        let args = [&self.first_book.to_str().unwrap(), &self.second_book.to_str().unwrap()];
        assert!(Command::new("mv").args(args).spawn().is_ok());
        sleep(Duration::from_millis(TIME_BETWEEN_TESTS));
      }
      TestMode::DirScan => {
        assert!(rename(&self.first_book, &self.second_book).is_ok());
        self.services.run_dir_scan(&self.tmp_dir.clone().to_string2());
      }
    };
    self.test_fn(&self.second_book.to_string2(), |book: &Book| assert_eq!(&SECOND_BOOK, &book.book_name));
  }
  pub fn move_second_book_to_first_dir(&mut self) {
    info!("File movement test: move second book to first dir");
    assert!(create_dir(&self.fist_dir).is_ok());
    let book_in_first_dir = self.tmp_dir.join(&FIRST_DIR).join(&SECOND_BOOK);
    match self.test_mode {
      TestMode::Notify => {
        let args = [&self.second_book.to_str().unwrap(), book_in_first_dir.parent().unwrap().to_str().unwrap()];
        assert!(Command::new("mv").args(args).spawn().is_ok());
        sleep(Duration::from_millis(TIME_BETWEEN_TESTS));
      }
      TestMode::DirScan => {
        assert!(rename(&self.second_book, &book_in_first_dir).is_ok());
        self.services.run_dir_scan(&self.tmp_dir.clone().to_string2());
      }
    }
    self.second_book = book_in_first_dir;
    self.test_fn(&self.second_book.to_string2(), |book: &Book| assert_eq!(&FIRST_DIR, &book.dir_name));
  }
  pub fn rename_first_dir_to_second(&mut self) {
    info!("Dir renaming test: rename_first_dir_to_second");
    assert!(rename(&self.fist_dir, &self.second_dir).is_ok());
    match self.test_mode {
      TestMode::Notify => { sleep(Duration::from_millis(TIME_BETWEEN_TESTS)); }
      TestMode::DirScan => { self.services.run_dir_scan(&self.tmp_dir.clone().to_string2()); }
    }

    self.second_book = self.tmp_dir.join(&SECOND_DIR).join(&SECOND_BOOK);
    self.test_fn(&self.second_book.to_string2(), |book: &Book| assert_eq!(&SECOND_DIR, &book.dir_name));
  }
  pub fn rename_second_book_to_first_in_second_dir(&mut self) {
    info!("File rename test2: rename second book to first in second dir");

    self.first_book = self.tmp_dir.join(&SECOND_DIR).join(&FIRST_BOOK);
    assert!(rename(&self.second_book, &self.first_book).is_ok());
    match self.test_mode {
      TestMode::Notify => { sleep(Duration::from_millis(TIME_BETWEEN_TESTS)); }
      TestMode::DirScan => { self.services.run_dir_scan(&self.tmp_dir.clone().to_string2()); }
    }

    self.test_fn(&self.first_book.to_string2(), |book: &Book| assert_eq!(&FIRST_BOOK, &book.book_name));
  }
  pub fn drop_second_dir(&mut self) {
    info!("Dir deletion test: drop_second_dir");
    assert!(remove_dir_all(&self.second_dir).is_ok());
    match self.test_mode {
      TestMode::Notify => { sleep(Duration::from_millis(TIME_BETWEEN_TESTS)); }
      TestMode::DirScan => { self.services.run_dir_scan(&self.tmp_dir.clone().to_string2()); }
    }
    assert_eq!(Book::get_by_path(&self.first_book.to_string2(), &self.db), None, "there shouldn't be a book");
  }
  pub fn drop_files(tmp_dir: &PathBuf) {
    debug!("Drop test files");
    match remove_dir_all(tmp_dir) {
      Ok(_) => {}
      Err(e) => error!("error when deleting tests_files_dir: {:?}", e),
    };
    match create_dir(tmp_dir) {
      Ok(_) => {}
      Err(e) => error!("error when creating tests_files_dir: {:?}", e),
    };
  }
  fn test_fn<F>(&self, book_path_in_db: &String, assert_fn: F) where F: Fn(&Book) {
    match Book::get_by_path(book_path_in_db, &self.db) {
      None => { panic!("book in db not found: {:?}", book_path_in_db) }
      Some(book) => { assert_fn(&book); }
    }
  }
  pub fn run_tests(&mut self) {
    Settings::set_path_to_scan_db_only(self.tmp_dir.to_string2(), None, &self.db);

    match self.test_mode {
      TestMode::Notify => { self.services.run_notify(self.tmp_dir.to_string2()).unwrap(); }
      TestMode::DirScan => {}
    }
    self.create_first_book();
    self.rename_first_book_to_second();
    self.move_second_book_to_first_dir();
    self.rename_first_dir_to_second();
    self.rename_second_book_to_first_in_second_dir();
    self.drop_second_dir();
    Self::drop_files(&self.tmp_dir);
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
  fn drop(&mut self) { self.services.stop(); }
}
